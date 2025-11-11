use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::{events::MouseData, logger::tracing, prelude::*, web::WebEventExt};
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCheck, LdChevronRight, LdCircle},
    Icon,
};
use wasm_bindgen::JsCast;
use web_sys::{window, DomRect, Element as WebElement, HtmlElement};

use crate::ui::custom::AppPortal;

/// Signal for managing dropdown menu open/close state and positioning metadata
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DropdownContext {
    pub open: bool,
    pub trigger_rect: Option<DropdownRect>,
    pub parent_rect: Option<DropdownRect>,
    pub overflow_rect: Option<DropdownRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DropdownRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl DropdownRect {
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
struct DropdownPlacement {
    top: f64,
    left: f64,
    origin: String,
    max_height: Option<f64>,
    max_width: Option<f64>,
}

static DROPDOWN_INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn next_dropdown_instance_id() -> usize {
    DROPDOWN_INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Properties for all dropdown menu components
#[derive(Props, PartialEq, Clone)]
pub struct DropdownMenuProps {
    /// Child elements to render inside the component
    children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    class: Option<String>,
}

/// Properties for DropdownMenuItem with variant support
#[derive(Props, PartialEq, Clone)]
pub struct DropdownMenuItemProps {
    children: Element,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    inset: bool,
    #[props(default = String::from("default"))]
    variant: String,
    #[props(default)]
    onclick: Option<EventHandler<MouseEvent>>,
}

/// Properties for checkbox and radio items
#[derive(Props, PartialEq, Clone)]
pub struct DropdownMenuToggleItemProps {
    children: Element,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    checked: bool,
    #[props(default)]
    onchange: Option<EventHandler<MouseEvent>>,
}

/// Root DropdownMenu component
#[component]
pub fn DropdownMenu(props: DropdownMenuProps) -> Element {
    use_context_provider(|| Signal::new(DropdownContext::default()));
    rsx! {
        div { class: "relative", "data-slot": "dropdown-menu", {props.children} }
    }
}

/// DropdownMenuTrigger component
#[component]
pub fn DropdownMenuTrigger(props: DropdownMenuProps) -> Element {
    let mut state = use_context::<Signal<DropdownContext>>();
    rsx! {
        button {
            "data-slot": "dropdown-menu-trigger",
            class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 data-[state=open]:bg-accent/50",
            onclick: move |e| {
                let next_open = !state.peek().open;
                if !next_open {
                    state.set(DropdownContext::default());
                    return;
                }
                let mouse_data: &MouseData = e.data.as_ref();
                let web_event = mouse_data.as_web_event();
                let target = web_event.target().and_then(|t| t.dyn_into::<WebElement>().ok());
                let Some(target_element) = target else {
                    let mut context = DropdownContext::default();
                    context.open = true;
                    state.set(context);
                    return;
                };
                let trigger_dom_rect = target_element.get_bounding_client_rect();
                let trigger_rect = DropdownRect::from_dom_rect(&trigger_dom_rect);
                tracing::info!(
                    "dropdown_trigger_rect tag={} rect=({:.1},{:.1},{:.1},{:.1}) screen=({:.1},{:.1},{:.1},{:.1})",
                    target_element.tag_name(), trigger_rect.x, trigger_rect.y, trigger_rect
                    .width, trigger_rect.height, trigger_dom_rect.x(), trigger_dom_rect.y(),
                    trigger_dom_rect.width(), trigger_dom_rect.height()
                );
                let parent_element = find_dropdown_root(&target_element);
                let parent_rect = parent_element
                    .as_ref()
                    .map(|el| DropdownRect::from_dom_rect(&el.get_bounding_client_rect()));
                tracing::info!(
                    "dropdown_parent_rect found={} rect={:?}", parent_element.is_some(),
                    parent_rect
                );
                let overflow_element = parent_element
                    .as_ref()
                    .and_then(|el| find_overflow_parent(el))
                    .or_else(|| find_overflow_parent(&target_element));
                tracing::info!(
                    "dropdown_overflow_parent found={} tag={}", overflow_element.is_some(),
                    overflow_element.as_ref().map(| el | el.tag_name()).unwrap_or_default()
                );
                let overflow_rect = overflow_element
                    .as_ref()
                    .map(|el| DropdownRect::from_dom_rect(&el.get_bounding_client_rect()))
                    .or_else(viewport_rect);
                tracing::info!("dropdown_overflow_rect={:?}", overflow_rect);
                let mut next_state = DropdownContext::default();
                next_state.open = true;
                next_state.trigger_rect = Some(trigger_rect);
                next_state.parent_rect = parent_rect
                    .or_else(|| {
                        tracing::debug!(
                            "Dropdown parent rect missing, falling back to trigger rect."
                        );
                        Some(trigger_rect)
                    });
                next_state.overflow_rect = overflow_rect;
                state.set(next_state);
            },
            {props.children}
        }
    }
}

/// DropdownMenuContent component
#[component]
pub fn DropdownMenuContent(props: DropdownMenuProps) -> Element {
    let mut context_signal = use_context::<Signal<DropdownContext>>();
    let state_snapshot = context_signal.read().clone();
    let is_open = state_snapshot.open;

    let content_id =
        use_signal(|| format!("dropdown-menu-content-{}", next_dropdown_instance_id()));
    let placement_state = use_signal(|| Option::<DropdownPlacement>::None);

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
                    "Missing rects for dropdown placement calculation: {:?}",
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
                            "dropdown_content_rect trigger={:?} parent={:?} container={:?} content=({:.1},{:.1})",
                            trigger_rect,
                            parent_rect,
                            container_rect,
                            element_rect.width(),
                            element_rect.height()
                        );
                        let placement = calculate_dropdown_placement(
                            trigger_rect,
                            parent_rect,
                            container_rect,
                            element_rect.width(),
                            element_rect.height(),
                        );

                        tracing::info!("dropdown_placement={:?}", placement);
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

    let mut class = vec![
        "bg-white dark:bg-zinc-950".to_string(),
        "dropdown-menu-content bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-10 min-w-[8rem]  rounded-md border p-1 absolute".to_string()
    ];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

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
                parts.push("overflow-y: auto".to_string());
            }

            if let Some(max_width) = placement.max_width {
                parts.push(format!("max-width: {:.2}px", max_width));
                parts.push("overflow-x: auto".to_string());
            }

            parts.join("; ")
        }
        (true, None) => "display: block; visibility: hidden;".to_string(),
    };
    let element_id = content_id.read().clone();

    rsx! {
        AppPortal {
            onclick: move |_| {
                context_signal.set(DropdownContext::default());
            },
            class: format!("{}", if is_open { "block" } else { "hidden" }).to_string(),
        }
        div {
            id: element_id,
            "data-slot": "dropdown-menu-content",
            "data-state": if is_open { "open" } else { "closed" },
            class: class.join(" "),
            style,
            {props.children}
        }
    }
}

/// DropdownMenuItem component
#[component]
pub fn DropdownMenuItem(props: DropdownMenuItemProps) -> Element {
    let mut open = use_context::<Signal<DropdownContext>>();
    let mut class = vec!["hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground data-[variant=destructive]:text-destructive data-[variant=destructive]:focus:bg-destructive/10 dark:data-[variant=destructive]:focus:bg-destructive/20 data-[variant=destructive]:focus:text-destructive data-[variant=destructive]:*:[svg]:!text-destructive [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4".to_string()];

    if props.inset {
        class.push("pl-8".to_string());
    }

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div {
            "data-slot": "dropdown-menu-item",
            "data-inset": props.inset.to_string(),
            "data-variant": props.variant,
            class: class.join(" "),
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
                open.set(DropdownContext::default());
            },
            {props.children}
        }
    }
}

/// DropdownMenuCheckboxItem component
#[component]
pub fn DropdownMenuCheckboxItem(props: DropdownMenuToggleItemProps) -> Element {
    let mut class = vec!["focus:bg-accent focus:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div {
            "data-slot": "dropdown-menu-checkbox-item",
            class: class.join(" "),
            onclick: move |e| {
                if let Some(handler) = &props.onchange {
                    handler.call(e);
                }
            },
            span { class: "pointer-events-none absolute left-2 flex size-3.5 items-center justify-center",
                if props.checked {
                    Icon { icon: LdCheck, class: "size-4" }
                }
            }
            {props.children}
        }
    }
}

/// DropdownMenuRadioItem component
#[component]
pub fn DropdownMenuRadioItem(props: DropdownMenuToggleItemProps) -> Element {
    let mut class = vec!["focus:bg-accent focus:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div {
            "data-slot": "dropdown-menu-radio-item",
            class: class.join(" "),
            onclick: move |e| {
                if let Some(handler) = &props.onchange {
                    handler.call(e);
                }
            },
            span { class: "pointer-events-none absolute left-2 flex size-3.5 items-center justify-center",
                if props.checked {
                    Icon { icon: LdCircle, class: "size-2" }
                }
            }
            {props.children}
        }
    }
}

/// DropdownMenuLabel component
#[component]
pub fn DropdownMenuLabel(props: DropdownMenuProps) -> Element {
    let mut class = vec!["px-2 py-1.5 text-sm font-medium".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dropdown-menu-label", class: class.join(" "), {props.children} }
    }
}

/// DropdownMenuSeparator component
#[component]
pub fn DropdownMenuSeparator(props: DropdownMenuProps) -> Element {
    let mut class = vec!["bg-border -mx-1 my-1 h-px".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dropdown-menu-separator", class: class.join(" "), {props.children} }
    }
}

/// DropdownMenuShortcut component
#[component]
pub fn DropdownMenuShortcut(props: DropdownMenuProps) -> Element {
    let mut class: Vec<String> =
        vec!["text-muted-foreground ml-auto text-xs tracking-widest".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        span { "data-slot": "dropdown-menu-shortcut", class: class.join(" "), {props.children} }
    }
}

/// DropdownMenuSubTrigger component
#[component]
pub fn DropdownMenuSubTrigger(props: DropdownMenuProps) -> Element {
    let mut class = vec!["focus:bg-accent focus:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground flex cursor-default items-center rounded-sm px-2 py-1.5 text-sm outline-hidden select-none".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dropdown-menu-sub-trigger", class: class.join(" "),
            {props.children}
            Icon { icon: LdChevronRight, class: "ml-auto size-4" }
        }
    }
}

/// DropdownMenuSubContent component
#[component]
pub fn DropdownMenuSubContent(props: DropdownMenuProps) -> Element {
    let mut class = vec!["bg-popover text-popover-foreground z-10 min-w-[8rem]  rounded-md border p-1 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dropdown-menu-sub-content", class: class.join(" "), {props.children} }
    }
}

fn find_dropdown_root(element: &WebElement) -> Option<WebElement> {
    let mut current = element.parent_element();
    while let Some(node) = current {
        if let Some(slot) = node.get_attribute("data-slot") {
            if slot == "dropdown-menu" {
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

fn viewport_rect() -> Option<DropdownRect> {
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

    Some(DropdownRect {
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

fn calculate_dropdown_placement(
    trigger: DropdownRect,
    parent: DropdownRect,
    container: DropdownRect,
    content_width: f64,
    content_height: f64,
) -> DropdownPlacement {
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

    DropdownPlacement {
        top,
        left,
        origin: format!("{} {}", origin_y, origin_x),
        max_height: max_height.and_then(normalize_dimension),
        max_width: max_width.and_then(normalize_dimension),
    }
}
