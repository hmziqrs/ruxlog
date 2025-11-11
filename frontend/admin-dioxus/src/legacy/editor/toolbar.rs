//! Toolbar component for the rich text editor.
//!
//! Provides formatting controls and block type selection.

use super::{commands::*, selection_in_code};
use crate::components::MediaPickerDialog;
use crate::store::{Media, MediaReference};
use dioxus::prelude::*;

/// Props for the Toolbar component.
#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    /// Callback when a command is executed.
    pub on_command: EventHandler<Box<dyn Command>>,

    /// Current selection state.
    pub selection: Selection,
}

/// Toolbar component with formatting controls.
#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut show_link_dialog = use_signal(|| false);
    let mut show_image_dialog = use_signal(|| false);
    let mut show_embed_dialog = use_signal(|| false);
    let mut show_table_dialog = use_signal(|| false);

    // Track active formatting states
    let mut is_bold = use_signal(|| false);
    let mut is_italic = use_signal(|| false);
    let mut is_underline = use_signal(|| false);
    let mut is_strikethrough = use_signal(|| false);
    let mut is_code = use_signal(|| false);

    // Update active states when selection changes
    use_effect(move || {
        if let Some(window) = web_sys::window() {
            if let Some(_document) = window.document() {
                // Check which formatting commands are active
                if let Ok(bold) = js_sys::eval("document.queryCommandState('bold')") {
                    is_bold.set(bold.as_bool().unwrap_or(false));
                }
                if let Ok(italic) = js_sys::eval("document.queryCommandState('italic')") {
                    is_italic.set(italic.as_bool().unwrap_or(false));
                }
                if let Ok(underline) = js_sys::eval("document.queryCommandState('underline')") {
                    is_underline.set(underline.as_bool().unwrap_or(false));
                }
                if let Ok(strike) = js_sys::eval("document.queryCommandState('strikeThrough')") {
                    is_strikethrough.set(strike.as_bool().unwrap_or(false));
                }

                is_code.set(selection_in_code(&_document));
            }
        }
    });

    rsx! {
        div {
            class: "toolbar border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 p-2 flex items-center gap-1 flex-wrap",
            role: "toolbar",
            "aria-label": "Text formatting toolbar",
            "aria-orientation": "horizontal",

            // Text formatting group
            div {
                class: "toolbar-group flex items-center gap-1 border-r border-gray-300 dark:border-gray-600 pr-2 mr-2",
                role: "group",
                "aria-label": "Text formatting",

                ToolbarButton {
                    icon: "B",
                    title: "Bold (Ctrl+B)",
                    active: *is_bold.read(),
                    on_click: move |_| {
                        props.on_command.call(Box::new(ToggleMark {
                            mark_type: MarkType::Bold,
                        }));
                    },
                }

                ToolbarButton {
                    icon: "I",
                    title: "Italic (Ctrl+I)",
                    class: "italic",
                    active: *is_italic.read(),
                    on_click: move |_| {
                        props.on_command.call(Box::new(ToggleMark {
                            mark_type: MarkType::Italic,
                        }));
                    },
                }

                ToolbarButton {
                    icon: "U",
                    title: "Underline (Ctrl+U)",
                    class: "underline",
                    active: *is_underline.read(),
                    on_click: move |_| {
                        props.on_command.call(Box::new(ToggleMark {
                            mark_type: MarkType::Underline,
                        }));
                    },
                }

                ToolbarButton {
                    icon: "S",
                    title: "Strikethrough",
                    class: "line-through",
                    active: *is_strikethrough.read(),
                    on_click: move |_| {
                        props.on_command.call(Box::new(ToggleMark {
                            mark_type: MarkType::Strike,
                        }));
                    },
                }

                ToolbarButton {
                    icon: "</>",
                    title: "Inline Code",
                    active: *is_code.read(),
                    on_click: move |_| {
                        props.on_command.call(Box::new(ToggleMark {
                            mark_type: MarkType::Code,
                        }));
                    },
                }
            }

            // Block type group
            div {
                class: "toolbar-group flex items-center gap-1 border-r border-gray-300 dark:border-gray-600 pr-2 mr-2",
                role: "group",
                "aria-label": "Block types and headings",

                ToolbarButton {
                    icon: "P",
                    title: "Paragraph",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::Paragraph,
                        }));
                    },
                }

                ToolbarButton {
                    icon: "H1",
                    title: "Heading 1",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::Heading { level: 1 },
                        }));
                    },
                }

                ToolbarButton {
                    icon: "H2",
                    title: "Heading 2",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::Heading { level: 2 },
                        }));
                    },
                }

                ToolbarButton {
                    icon: "H3",
                    title: "Heading 3",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::Heading { level: 3 },
                        }));
                    },
                }
            }

            // List group
            div {
                class: "toolbar-group flex items-center gap-1 border-r border-gray-300 dark:border-gray-600 pr-2 mr-2",
                role: "group",
                "aria-label": "Lists",

                ToolbarButton {
                    icon: "•",
                    title: "Bullet List",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::BulletList { items: vec![] },
                        }));
                    },
                }

                ToolbarButton {
                    icon: "1.",
                    title: "Numbered List",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::OrderedList { items: vec![] },
                        }));
                    },
                }

                ToolbarButton {
                    icon: "☐",
                    title: "Task List",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::TaskList { items: vec![] },
                        }));
                    },
                }
            }

            // Insert group
            div {
                class: "toolbar-group flex items-center gap-1 border-r border-gray-300 dark:border-gray-600 pr-2 mr-2",

                ToolbarButton {
                    icon: "\"",
                    title: "Quote",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::Quote,
                        }));
                    },
                }

                ToolbarButton {
                    icon: "Code",
                    title: "Code Block",
                    on_click: move |_| {
                        props.on_command.call(Box::new(SetBlockType {
                            block_type: super::ast::BlockKind::CodeBlock {
                                language: None,
                                code: String::new(),
                            },
                        }));
                    },
                }

                ToolbarButton {
                    icon: "—",
                    title: "Horizontal Rule",
                    on_click: move |_| {
                        props.on_command.call(Box::new(InsertBlock {
                            block_type: super::ast::BlockKind::Rule,
                        }));
                    },
                }
            }

            // Media group
            div {
                class: "toolbar-group flex items-center gap-1",

                ToolbarButton {
                    icon: "🔗",
                    title: "Insert Link",
                    on_click: move |_| {
                        show_link_dialog.set(true);
                    },
                }

                ToolbarButton {
                    icon: "📷",
                    title: "Insert Image",
                    on_click: move |_| {
                        show_image_dialog.set(!show_image_dialog());
                    },
                }

                ToolbarButton {
                    icon: "▶",
                    title: "Embed Media",
                    on_click: move |_| {
                        show_embed_dialog.set(true);
                    },
                }

                ToolbarButton {
                    icon: "▦",
                    title: "Insert Table",
                    on_click: move |_| {
                        show_table_dialog.set(true);
                    },
                }
            }

            // Dialogs
            if *show_link_dialog.read() {
                LinkDialog {
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

            MediaPickerDialog {
                open: *show_image_dialog.read(),
                on_close: move |_| show_image_dialog.set(false),
                on_select: move |media: Media| {
                    props.on_command.call(Box::new(InsertBlock {
                        block_type: super::ast::BlockKind::Image {
                            src: media.file_url.clone(),
                            alt: Some(media.object_key.clone()),
                            title: None,
                            width: None,
                            height: None,
                            caption: None,
                        },
                    }));
                },
                filter_type: Some("image/*".to_string()),
                reference_type: Some(MediaReference::Post),
            }

            if *show_embed_dialog.read() {
                EmbedDialog {
                    on_close: move |_| show_embed_dialog.set(false),
                    on_insert: move |(provider, url)| {
                        props.on_command.call(Box::new(InsertBlock {
                            block_type: super::ast::BlockKind::Embed {
                                provider,
                                url,
                                title: None,
                                width: None,
                                height: None,
                            },
                        }));
                        show_embed_dialog.set(false);
                    },
                }
            }

            if *show_table_dialog.read() {
                TableDialog {
                    on_close: move |_| show_table_dialog.set(false),
                    on_insert: move |(rows, cols)| {
                        gloo_console::log!("[Toolbar] TableDialog on_insert called: rows=", rows, "cols=", cols);
                        props.on_command.call(Box::new(super::commands::InsertTable {
                            rows,
                            cols,
                        }));
                        gloo_console::log!("[Toolbar] Closing table dialog");
                        show_table_dialog.set(false);
                    },
                }
            }
        }
    }
}

/// Individual toolbar button component.
#[component]
fn ToolbarButton(
    icon: String,
    title: String,
    #[props(default = String::new())] class: String,
    #[props(default = false)] active: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let active_class = if active {
        "bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300"
    } else {
        ""
    };

    // Extract the action name from title (before the shortcut in parentheses)
    let aria_label = title.split('(').next().unwrap_or(&title).trim();

    rsx! {
        button {
            class: "toolbar-button px-3 py-1.5 rounded text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 active:bg-gray-300 dark:active:bg-gray-600 transition-colors text-sm font-medium {class} {active_class}",
            title: "{title}",
            r#type: "button",
            role: "button",
            "aria-label": "{aria_label}",
            "aria-pressed": "{active}",
            tabindex: "0",
            onclick: move |evt| on_click.call(evt),
            "{icon}"
        }
    }
}

/// Link insertion dialog.
#[component]
#[component]
pub fn LinkDialog(
    on_close: EventHandler<Event<MouseData>>,
    on_insert: EventHandler<(String, Option<String>)>,
) -> Element {
    let mut href = use_signal(|| String::new());
    let mut title = use_signal(|| String::new());

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 dark:bg-opacity-70 flex items-center justify-center z-50",
            onclick: move |evt| {
                evt.stop_propagation();
                on_close.call(evt);
            },

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100", "Insert Link" }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "URL" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "url",
                            placeholder: "https://example.com",
                            value: "{href}",
                            oninput: move |evt| href.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Title (optional)" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "text",
                            placeholder: "Link title",
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                        }
                    }

                    div { class: "flex justify-end gap-2 pt-2",
                        button {
                            class: "px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md",
                            r#type: "button",
                            onclick: move |evt| on_close.call(evt),
                            "Cancel"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 dark:bg-blue-700 text-white rounded-md hover:bg-blue-700 dark:hover:bg-blue-600",
                            r#type: "button",
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

/// Image insertion dialog.
#[component]
fn ImageDialog(
    on_close: EventHandler<MouseEvent>,
    on_insert: EventHandler<(String, Option<String>, Option<String>)>,
) -> Element {
    let mut src = use_signal(|| String::new());
    let mut alt = use_signal(|| String::new());
    let mut caption = use_signal(|| String::new());

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 dark:bg-opacity-70 flex items-center justify-center z-50",
            onclick: move |evt| {
                evt.stop_propagation();
                on_close.call(evt);
            },

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100", "Insert Image" }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Image URL" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "url",
                            placeholder: "https://example.com/image.jpg",
                            value: "{src}",
                            oninput: move |evt| src.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Alt Text" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "text",
                            placeholder: "Description of the image",
                            value: "{alt}",
                            oninput: move |evt| alt.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Caption (optional)" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "text",
                            placeholder: "Image caption",
                            value: "{caption}",
                            oninput: move |evt| caption.set(evt.value()),
                        }
                    }

                    div { class: "flex justify-end gap-2 pt-2",
                        button {
                            class: "px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md",
                            r#type: "button",
                            onclick: move |evt| on_close.call(evt),
                            "Cancel"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 dark:bg-blue-700 text-white rounded-md hover:bg-blue-700 dark:hover:bg-blue-600",
                            r#type: "button",
                            onclick: move |_| {
                                let alt_val = if alt.read().is_empty() {
                                    None
                                } else {
                                    Some(alt.read().clone())
                                };
                                let caption_val = if caption.read().is_empty() {
                                    None
                                } else {
                                    Some(caption.read().clone())
                                };
                                on_insert.call((src.read().clone(), alt_val, caption_val));
                            },
                            "Insert"
                        }
                    }
                }
            }
        }
    }
}

/// Embed media dialog.
#[component]
fn EmbedDialog(
    on_close: EventHandler<MouseEvent>,
    on_insert: EventHandler<(super::ast::EmbedProvider, String)>,
) -> Element {
    let mut url = use_signal(|| String::new());
    let mut provider = use_signal(|| super::ast::EmbedProvider::Youtube);

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 dark:bg-opacity-70 flex items-center justify-center z-50",
            onclick: move |evt| {
                evt.stop_propagation();
                on_close.call(evt);
            },

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100", "Embed Media" }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Provider" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            onchange: move |evt| {
                                let val = evt.value();
                                provider.set(match val.as_str() {
                                    "youtube" => super::ast::EmbedProvider::Youtube,
                                    "x" => super::ast::EmbedProvider::X,
                                    _ => super::ast::EmbedProvider::Generic,
                                });
                            },
                            option { value: "youtube", "YouTube" }
                            option { value: "x", "X (Twitter)" }
                            option { value: "generic", "Generic" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "URL" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "url",
                            placeholder: "https://youtube.com/watch?v=...",
                            value: "{url}",
                            oninput: move |evt| url.set(evt.value()),
                        }
                    }

                    div { class: "flex justify-end gap-2 pt-2",
                        button {
                            class: "px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md",
                            r#type: "button",
                            onclick: move |evt| on_close.call(evt),
                            "Cancel"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 dark:bg-blue-700 text-white rounded-md hover:bg-blue-700 dark:hover:bg-blue-600",
                            r#type: "button",
                            onclick: move |_| {
                                on_insert.call((provider.read().clone(), url.read().clone()));
                            },
                            "Insert"
                        }
                    }
                }
            }
        }
    }
}

/// Table insertion dialog.
#[component]
pub fn TableDialog(
    on_close: EventHandler<MouseEvent>,
    on_insert: EventHandler<(usize, usize)>,
) -> Element {
    let mut rows = use_signal(|| String::from("3"));
    let mut cols = use_signal(|| String::from("3"));

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 dark:bg-opacity-70 flex items-center justify-center z-50",
            onclick: move |evt| {
                evt.stop_propagation();
                on_close.call(evt);
            },

            div {
                class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100", "Insert Table" }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Rows" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "number",
                            min: "1",
                            max: "20",
                            value: "{rows}",
                            oninput: move |evt| rows.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1", "Columns" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            r#type: "number",
                            min: "1",
                            max: "10",
                            value: "{cols}",
                            oninput: move |evt| cols.set(evt.value()),
                        }
                    }

                    div { class: "flex justify-end gap-2 pt-2",
                        button {
                            class: "px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md",
                            r#type: "button",
                            onclick: move |evt| on_close.call(evt),
                            "Cancel"
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 dark:bg-blue-700 text-white rounded-md hover:bg-blue-700 dark:hover:bg-blue-600",
                            r#type: "button",
                            onclick: move |_| {
                                let rows_val = rows.read().parse::<usize>().unwrap_or(3).clamp(1, 20);
                                let cols_val = cols.read().parse::<usize>().unwrap_or(3).clamp(1, 10);
                                gloo_console::log!("[TableDialog] Insert clicked: rows=", rows_val, "cols=", cols_val);
                                on_insert.call((rows_val, cols_val));
                            },
                            "Insert"
                        }
                    }
                }
            }
        }
    }
}
