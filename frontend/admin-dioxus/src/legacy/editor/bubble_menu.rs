//! Bubble menu component for contextual formatting on text selection.
//!
//! Displays a floating toolbar near selected text with quick formatting actions.

use super::commands::*;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// Props for the BubbleMenu component.
#[derive(Props, Clone, PartialEq)]
pub struct BubbleMenuProps {
    /// Whether the menu should be shown
    pub show: bool,
    /// Current selection state
    pub selection: Selection,
    /// Callback when a command is executed
    pub on_command: EventHandler<Box<dyn Command>>,
    /// ID of the editor element for positioning
    pub editor_id: String,
}

/// Bubble menu component with contextual formatting controls.
#[component]
pub fn BubbleMenu(props: BubbleMenuProps) -> Element {
    let mut position = use_signal(|| BubblePosition {
        top: 0.0,
        left: 0.0,
    });
    let mut is_bold = use_signal(|| false);
    let mut is_italic = use_signal(|| false);
    let mut is_underline = use_signal(|| false);
    let mut is_code = use_signal(|| false);
    let mut show_link_dialog = use_signal(|| false);

    // Update position and active states when selection changes or show changes
    let selection_clone = props.selection.clone();
    let show_clone = props.show;
    let editor_id_clone = props.editor_id.clone();
    use_effect(move || {
        let has_selection = !selection_clone.is_collapsed();
        if show_clone && has_selection {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // Update active formatting states
                    if let Ok(bold) = js_sys::eval("document.queryCommandState('bold')") {
                        is_bold.set(bold.as_bool().unwrap_or(false));
                    }
                    if let Ok(italic) = js_sys::eval("document.queryCommandState('italic')") {
                        is_italic.set(italic.as_bool().unwrap_or(false));
                    }
                    if let Ok(underline) = js_sys::eval("document.queryCommandState('underline')") {
                        is_underline.set(underline.as_bool().unwrap_or(false));
                    }

                    // Check if in code
                    is_code.set(super::selection_in_code(&document));

                    // Calculate position
                    if let Some(sel) = window.get_selection().ok().flatten() {
                        if sel.range_count() > 0 {
                            if let Ok(range) = sel.get_range_at(0) {
                                let rect = range.get_bounding_client_rect();
                                {
                                    // Get editor element to calculate relative position
                                    if let Some(editor) =
                                        document.get_element_by_id(&editor_id_clone)
                                    {
                                        if let Ok(editor_html) = editor.dyn_into::<HtmlElement>() {
                                            let _editor_rect =
                                                editor_html.get_bounding_client_rect();

                                            // Position menu above selection, centered
                                            let menu_width = 300.0; // Approximate menu width
                                            let menu_height = 44.0; // Approximate menu height
                                            let gap = 8.0; // Gap between selection and menu

                                            let left = rect.left() + (rect.width() / 2.0)
                                                - (menu_width / 2.0);
                                            let top = rect.top() - menu_height - gap;

                                            // Adjust if menu would go off-screen
                                            let left = left.max(10.0).min(
                                                window
                                                    .inner_width()
                                                    .ok()
                                                    .and_then(|w| w.as_f64())
                                                    .unwrap_or(1000.0)
                                                    - menu_width
                                                    - 10.0,
                                            );
                                            let top = if top < 0.0 {
                                                rect.bottom() + gap // Show below if not enough space above
                                            } else {
                                                top
                                            };

                                            position.set(BubblePosition { top, left });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let has_selection = !props.selection.is_collapsed();
    if !props.show || !has_selection {
        return rsx! {};
    }

    let pos = position();

    rsx! {
        div {
            class: "bubble-menu fixed z-50 bg-base-100 dark:bg-gray-800 border border-base-300 dark:border-gray-600 rounded-lg shadow-lg p-1 flex items-center gap-1",
            style: "top: {pos.top}px; left: {pos.left}px; transition: opacity 0.2s ease;",
            onclick: move |e| e.stop_propagation(),

            // Bold
            BubbleMenuButton {
                icon: "B",
                title: "Bold",
                class: "font-bold",
                active: *is_bold.read(),
                on_click: move |_| {
                    props.on_command.call(Box::new(ToggleMark {
                        mark_type: MarkType::Bold,
                    }));
                },
            }

            // Italic
            BubbleMenuButton {
                icon: "I",
                title: "Italic",
                class: "italic",
                active: *is_italic.read(),
                on_click: move |_| {
                    props.on_command.call(Box::new(ToggleMark {
                        mark_type: MarkType::Italic,
                    }));
                },
            }

            // Underline
            BubbleMenuButton {
                icon: "U",
                title: "Underline",
                class: "underline",
                active: *is_underline.read(),
                on_click: move |_| {
                    props.on_command.call(Box::new(ToggleMark {
                        mark_type: MarkType::Underline,
                    }));
                },
            }

            // Strikethrough
            BubbleMenuButton {
                icon: "S",
                title: "Strikethrough",
                class: "line-through",
                active: false,
                on_click: move |_| {
                    props.on_command.call(Box::new(ToggleMark {
                        mark_type: MarkType::Strike,
                    }));
                },
            }

            // Separator
            div { class: "w-px h-6 bg-base-300 dark:bg-gray-600 mx-1" }

            // Code
            BubbleMenuButton {
                icon: "</>",
                title: "Inline Code",
                class: "font-mono text-xs",
                active: *is_code.read(),
                on_click: move |_| {
                    props.on_command.call(Box::new(ToggleMark {
                        mark_type: MarkType::Code,
                    }));
                },
            }

            // Link
            BubbleMenuButton {
                icon: "🔗",
                title: "Insert Link",
                active: false,
                on_click: move |_| {
                    show_link_dialog.set(true);
                },
            }

            // Link dialog
            if *show_link_dialog.read() {
                LinkDialogBubble {
                    on_close: move |_| show_link_dialog.set(false),
                    on_insert: move |(href, title)| {
                        props.on_command.call(Box::new(InsertLink {
                            href,
                            title,
                            target_blank: true,
                        }));
                        show_link_dialog.set(false);
                    },
                }
            }
        }
    }
}

/// Position for the bubble menu.
#[derive(Clone, Copy, PartialEq)]
struct BubblePosition {
    top: f64,
    left: f64,
}

/// Button component for the bubble menu.
#[component]
fn BubbleMenuButton(
    icon: String,
    title: String,
    #[props(default = String::new())] class: String,
    active: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let button_class = if active {
        format!(
            "btn btn-sm btn-ghost bg-primary/20 text-primary hover:bg-primary/30 min-h-0 h-8 px-2 {}",
            class
        )
    } else {
        format!(
            "btn btn-sm btn-ghost hover:bg-base-200 dark:hover:bg-gray-700 min-h-0 h-8 px-2 {}",
            class
        )
    };

    rsx! {
        button {
            class: "{button_class}",
            title: "{title}",
            r#type: "button",
            onclick: move |evt| on_click.call(evt),
            "{icon}"
        }
    }
}

/// Mini link dialog for bubble menu.
#[component]
fn LinkDialogBubble(
    on_close: EventHandler<Event<MouseData>>,
    on_insert: EventHandler<(String, Option<String>)>,
) -> Element {
    let mut href = use_signal(|| String::new());
    let mut title = use_signal(|| String::new());

    rsx! {
        div {
            class: "fixed inset-0 bg-black/20 dark:bg-black/40 flex items-center justify-center z-[60]",
            onclick: move |evt| {
                evt.stop_propagation();
                on_close.call(evt);
            },

            div {
                class: "bg-base-100 dark:bg-gray-800 rounded-lg p-4 shadow-xl max-w-sm w-full mx-4",
                onclick: move |evt| evt.stop_propagation(),

                h4 { class: "text-sm font-semibold mb-3 text-base-content dark:text-gray-100",
                    "Insert Link"
                }

                div { class: "space-y-3",
                    div {
                        label { class: "block text-xs font-medium text-base-content/70 dark:text-gray-300 mb-1",
                            "URL"
                        }
                        input {
                            class: "input input-sm input-bordered w-full bg-base-100 dark:bg-gray-700",
                            r#type: "url",
                            placeholder: "https://example.com",
                            value: "{href}",
                            oninput: move |evt| href.set(evt.value()),
                            autofocus: true,
                        }
                    }

                    div {
                        label { class: "block text-xs font-medium text-base-content/70 dark:text-gray-300 mb-1",
                            "Title (optional)"
                        }
                        input {
                            class: "input input-sm input-bordered w-full bg-base-100 dark:bg-gray-700",
                            r#type: "text",
                            placeholder: "Link title",
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                        }
                    }

                    div { class: "flex justify-end gap-2 pt-1",
                        button {
                            class: "btn btn-sm btn-ghost",
                            r#type: "button",
                            onclick: move |evt| on_close.call(evt),
                            "Cancel"
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            r#type: "button",
                            disabled: href.read().is_empty(),
                            onclick: move |_| {
                                let title_val = if title.read().is_empty() {
                                    None
                                } else {
                                    Some(title.read().clone())
                                };
                                on_insert.call((href.read().clone(), title_val));
                            },
                            "Insert"
                        }
                    }
                }
            }
        }
    }
}
