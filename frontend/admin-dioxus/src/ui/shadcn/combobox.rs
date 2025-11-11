use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::{events::MouseData, logger::tracing, prelude::*, web::WebEventExt};
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCheck, LdChevronsUpDown},
    Icon,
};
use wasm_bindgen::JsCast;
use web_sys::{window, DomRect, Element as WebElement, HtmlElement};

use crate::ui::custom::AppPortal;

/// Signal for managing combobox open/close state and positioning metadata
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ComboboxContext {
    pub open: bool,
    pub trigger_rect: Option<ComboboxRect>,
    pub parent_rect: Option<ComboboxRect>,
    pub overflow_rect: Option<ComboboxRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ComboboxRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl ComboboxRect {
    fn from_dom_rect(rect: &DomRect) -> Self {
        Self {
            x: rect.x(),
            y: rect.y(),
            width: rect.width(),
            height: rect.height(),
        }
    }

    fn right(&self) -> f64 {
        self.x + self.width
    }

    fn bottom(&self) -> f64 {
        self.y + self.height
    }

    fn left(&self) -> f64 {
        self.x
    }

    fn top(&self) -> f64 {
        self.y
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ComboboxPlacement {
    top: f64,
    left: f64,
    origin: String,
    max_height: Option<f64>,
    max_width: Option<f64>,
}

static COMBOBOX_INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn next_combobox_instance_id() -> usize {
    COMBOBOX_INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComboboxItem {
    pub value: String,
    pub label: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct ComboboxProps {
    /// List of items to display in the combobox
    pub items: Vec<ComboboxItem>,
    /// Placeholder text for the combobox trigger
    #[props(default = String::from("Select an option..."))]
    pub placeholder: String,
    /// Placeholder text for the search input
    #[props(default = String::from("Search..."))]
    pub search_placeholder: String,
    /// Empty message when no items match the search
    #[props(default = String::from("No items found."))]
    pub empty_message: String,
    /// The currently selected value
    #[props(default)]
    pub value: Option<String>,
    /// Callback when the value changes
    #[props(default)]
    pub onvaluechange: Option<EventHandler<Option<String>>>,
    /// Width of the combobox
    #[props(default = String::from("w-[200px]"))]
    pub width: String,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn Combobox(props: ComboboxProps) -> Element {
    let mut value = use_signal(|| props.value.clone());
    let mut query = use_signal(|| String::new());
    let mut context_signal = use_signal(|| ComboboxContext::default());
    let content_id = use_signal(|| format!("combobox-content-{}", next_combobox_instance_id()));
    let placement_state = use_signal(|| Option::<ComboboxPlacement>::None);

    // Handle incoming prop changes
    use_effect(move || {
        if props.value != *value.peek() {
            value.set(props.value.clone());
        }
    });

    // Calculate placement when opened
    {
        let element_id = content_id.read().clone();
        let mut placement_state = placement_state.clone();
        let context_signal = context_signal.clone();

        use_effect(move || {
            let state = context_signal.read().clone();
            if !state.open {
                placement_state.set(None);
                return;
            }

            let (Some(trigger_rect), Some(parent_rect), Some(container_rect)) =
                (state.trigger_rect, state.parent_rect, state.overflow_rect)
            else {
                tracing::debug!(
                    "Missing rects for combobox placement calculation: {:?}",
                    state
                );
                return;
            };

            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Some(element) = document.get_element_by_id(&element_id) {
                        let html_element = element.dyn_ref::<HtmlElement>();
                        if let Some(el) = html_element {
                            let style = el.style();
                            let _ = style.set_property("visibility", "hidden");
                            let _ = style.set_property("display", "block");
                        }

                        let element_rect = element.get_bounding_client_rect();
                        tracing::info!(
                            "combobox_content_rect trigger={:?} parent={:?} container={:?} content=({:.1},{:.1})",
                            trigger_rect,
                            parent_rect,
                            container_rect,
                            element_rect.width(),
                            element_rect.height()
                        );
                        let placement = calculate_combobox_placement(
                            trigger_rect,
                            parent_rect,
                            container_rect,
                            element_rect.width(),
                            element_rect.height(),
                        );

                        tracing::info!("combobox_placement={:?}", placement);
                        placement_state.set(Some(placement));

                        if let Some(el) = html_element {
                            let style = el.style();
                            let _ = style.remove_property("visibility");
                        }
                    }
                }
            }
        });
    }

    // Find the selected item to display in the trigger
    let selected_item = props
        .items
        .iter()
        .find(|item| Some(&item.value) == value.read().as_ref());

    let display_text = selected_item
        .map(|item| item.label.clone())
        .unwrap_or(props.placeholder.clone());

    let mut class = vec!["relative".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    // Precompute filtered items based on query
    let q = query.read().to_lowercase();
    let filtered: Vec<ComboboxItem> = props
        .items
        .iter()
        .filter(|item| {
            q.is_empty()
                || item.label.to_lowercase().contains(&q)
                || item.value.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    let mapped: Vec<(String, String, bool)> = filtered
        .iter()
        .map(|item| {
            let val = item.value.clone();
            let label = item.label.clone();
            let sel = Some(&val) == value.read().as_ref();
            (val, label, sel)
        })
        .collect();

    let state_snapshot = context_signal.read().clone();
    let is_open = state_snapshot.open;
    let element_id = content_id.read().clone();
    let placement_style = placement_state.read().clone();
    let style = match (is_open, placement_style) {
        (false, _) => "display: none;".to_string(),
        (true, Some(placement)) => {
            let mut parts = vec![
                "display: block".to_string(),
                format!("top: {:.2}px", placement.top),
                format!("left: {:.2}px", placement.left),
                format!("transform-origin: {}", placement.origin),
            ];

            if let Some(max_height) = placement.max_height {
                parts.push(format!("max-height: {:.2}px", max_height));
            }

            if let Some(max_width) = placement.max_width {
                parts.push(format!("max-width: {:.2}px", max_width));
            }

            parts.join("; ")
        }
        (true, None) => "display: block; visibility: hidden;".to_string(),
    };

    rsx! {
        div { class: class.join(" "), "data-slot": "combobox",
            button {
                class: format!("{} bg-background hover:bg-accent hover:text-accent-foreground inline-flex h-10 w-full items-center justify-between rounded-md border border-input px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50", props.width),
                role: "combobox",
                "data-slot": "combobox-trigger",
                "aria-expanded": if is_open { "true" } else { "false" },
                onclick: move |e| {
                    let next_open = !context_signal.peek().open;
                    if !next_open {
                        context_signal.set(ComboboxContext::default());
                        query.set(String::new());
                        return;
                    }
                    let mouse_data: &MouseData = e.data.as_ref();
                    let web_event = mouse_data.as_web_event();
                    let target = web_event.target().and_then(|t| t.dyn_into::<WebElement>().ok());
                    let Some(target_element) = target else {
                        let mut context = ComboboxContext::default();
                        context.open = true;
                        context_signal.set(context);
                        return;
                    };
                    let trigger_dom_rect = target_element.get_bounding_client_rect();
                    let trigger_rect = ComboboxRect::from_dom_rect(&trigger_dom_rect);
                    tracing::info!(
                        "combobox_trigger_rect tag={} rect=({:.1},{:.1},{:.1},{:.1})",
                        target_element.tag_name(), trigger_rect.x, trigger_rect.y, trigger_rect.width, trigger_rect.height
                    );
                    let parent_element = find_combobox_root(&target_element);
                    let parent_rect = parent_element
                        .as_ref()
                        .map(|el| ComboboxRect::from_dom_rect(&el.get_bounding_client_rect()));
                    tracing::info!("combobox_parent_rect found={} rect={:?}", parent_element.is_some(), parent_rect);
                    let overflow_element = parent_element
                        .as_ref()
                        .and_then(|el| find_overflow_parent(el))
                        .or_else(|| find_overflow_parent(&target_element));
                    tracing::info!(
                        "combobox_overflow_parent found={} tag={}",
                        overflow_element.is_some(),
                        overflow_element.as_ref().map(|el| el.tag_name()).unwrap_or_default()
                    );
                    let overflow_rect = overflow_element
                        .as_ref()
                        .map(|el| ComboboxRect::from_dom_rect(&el.get_bounding_client_rect()))
                        .or_else(viewport_rect);
                    tracing::info!("combobox_overflow_rect={:?}", overflow_rect);
                    let mut next_state = ComboboxContext::default();
                    next_state.open = true;
                    next_state.trigger_rect = Some(trigger_rect);
                    next_state.parent_rect = parent_rect.or_else(|| {
                        tracing::debug!("Combobox parent rect missing, falling back to trigger rect.");
                        Some(trigger_rect)
                    });
                    next_state.overflow_rect = overflow_rect;
                    context_signal.set(next_state);
                },
                span { class: "truncate", "{display_text}" }
                Icon {
                    icon: LdChevronsUpDown,
                    class: "ml-2 h-4 w-4 shrink-0 opacity-50",
                }
            }

            // Backdrop portal
            if is_open {
                AppPortal {
                    onclick: move |_| {
                        context_signal.set(ComboboxContext::default());
                        query.set(String::new());
                    },
                    class: "block".to_string(),
                }
            }

            // Content
            div {
                id: element_id,
                "data-slot": "combobox-content",
                "data-state": if is_open { "open" } else { "closed" },
                class: format!("{} bg-popover text-popover-foreground z-10 rounded-md border p-0 absolute data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95", props.width),
                style,
                div { class: "p-2",
                    input {
                        class: "mb-2 h-9 w-full rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring",
                        r#type: "text",
                        placeholder: props.search_placeholder.clone(),
                        value: query.read().clone(),
                        oninput: move |e| {
                            query.set(e.value());
                        }
                    }
                    div { class: "max-h-64 overflow-auto",
                        if mapped.is_empty() {
                            div { class: "px-3 py-2 text-sm text-muted-foreground", "{props.empty_message}" }
                        } else {
                            for (item_value, item_label, is_selected) in mapped {
                                button {
                                    class: "flex w-full items-center justify-between rounded-sm px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground",
                                    onclick: move |_| {
                                        let new_value = if is_selected { None } else { Some(item_value.clone()) };
                                        value.set(new_value.clone());
                                        if let Some(handler) = &props.onvaluechange {
                                            handler.call(new_value);
                                        }
                                        context_signal.set(ComboboxContext::default());
                                        query.set(String::new());
                                    },
                                    span { class: "truncate", "{item_label}" }
                                    if is_selected { Icon { icon: LdCheck, class: "ml-2 h-4 w-4" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn find_combobox_root(element: &WebElement) -> Option<WebElement> {
    let mut current = element.parent_element();
    while let Some(node) = current {
        if let Some(slot) = node.get_attribute("data-slot") {
            if slot == "combobox" {
                return Some(node);
            }
        }
        current = node.parent_element();
    }
    None
}

fn find_overflow_parent(element: &WebElement) -> Option<WebElement> {
    let window = window()?;
    let mut current = element.parent_element();
    while let Some(node) = current {
        if let Ok(Some(style)) = window.get_computed_style(&node) {
            let overflow = style.get_property_value("overflow").ok();
            let overflow_x = style.get_property_value("overflow-x").ok();
            let overflow_y = style.get_property_value("overflow-y").ok();

            if overflow
                .as_ref()
                .map(|value| is_scrollable_value(value))
                .unwrap_or(false)
                || overflow_x
                    .as_ref()
                    .map(|value| is_scrollable_value(value))
                    .unwrap_or(false)
                || overflow_y
                    .as_ref()
                    .map(|value| is_scrollable_value(value))
                    .unwrap_or(false)
            {
                return Some(node);
            }
        }
        current = node.parent_element();
    }
    None
}

fn is_scrollable_value(value: &str) -> bool {
    matches!(value.trim(), "auto" | "scroll" | "hidden" | "clip")
}

fn viewport_rect() -> Option<ComboboxRect> {
    let window = window()?;
    let width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Some(ComboboxRect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    })
}

fn normalize_dimension(value: f64) -> Option<f64> {
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

fn calculate_combobox_placement(
    trigger: ComboboxRect,
    parent: ComboboxRect,
    container: ComboboxRect,
    content_width: f64,
    content_height: f64,
) -> ComboboxPlacement {
    const GAP: f64 = 4.0;

    let space_above = trigger.top() - container.top();
    let space_below = container.bottom() - trigger.bottom();

    let fits_below = content_height + GAP <= space_below;
    let fits_above = content_height + GAP <= space_above;

    let show_below = if fits_below {
        true
    } else if fits_above {
        false
    } else {
        space_below >= space_above
    };

    let mut origin_y = if show_below { "top" } else { "bottom" };
    let mut top_vp = if show_below {
        trigger.bottom() + GAP
    } else {
        trigger.top() - content_height - GAP
    };

    let container_top_bound = container.top() + GAP;
    let container_bottom_bound = container.bottom() - GAP;
    let mut max_height = None;

    if show_below && !fits_below && space_below > 0.0 {
        max_height = Some(space_below - GAP);
    } else if !show_below && !fits_above && space_above > 0.0 {
        max_height = Some(space_above - GAP);
    }

    if top_vp < container_top_bound {
        top_vp = container_top_bound;
        origin_y = "top";
    }

    if top_vp + content_height > container_bottom_bound {
        let available = (container_bottom_bound - top_vp).max(0.0);
        if available < content_height && available > 0.0 {
            max_height = Some(available);
        }
        top_vp = (container_bottom_bound - content_height).max(container_top_bound);
        if top_vp < container_top_bound {
            top_vp = container_top_bound;
        }
    }

    let container_left_bound = container.left() + GAP;
    let container_right_bound = container.right() - GAP;
    let mut max_width = None;
    let mut origin_x = "left";

    let available_width = (container_right_bound - container_left_bound).max(0.0);
    let mut effective_content_width = content_width;
    if effective_content_width > available_width && available_width > 0.0 {
        max_width = Some(available_width);
        effective_content_width = available_width;
    }

    let space_right = container_right_bound - trigger.left();
    let space_left = trigger.right() - container_left_bound;

    let align_left = if effective_content_width <= space_right {
        true
    } else if effective_content_width <= space_left {
        false
    } else {
        space_right >= space_left
    };

    let mut left_vp = if align_left {
        trigger.left()
    } else {
        origin_x = "right";
        trigger.right() - effective_content_width
    };

    if left_vp < container_left_bound {
        left_vp = container_left_bound;
        origin_x = "left";
    }

    if left_vp + effective_content_width > container_right_bound {
        left_vp = (container_right_bound - effective_content_width).max(container_left_bound);
        origin_x = if align_left { "left" } else { "right" };
    }

    let top = top_vp - parent.top();
    let left = left_vp - parent.left();

    ComboboxPlacement {
        top,
        left,
        origin: format!("{} {}", origin_y, origin_x),
        max_height: max_height.and_then(normalize_dimension),
        max_width: max_width.and_then(normalize_dimension),
    }
}
