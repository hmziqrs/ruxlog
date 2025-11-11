//! Rich text editor demo screen.
//!
//! This screen demonstrates the usage of the rich text editor component.

use crate::components::editor::{ContentViewer, RichTextEditor, SimpleEditor};
use dioxus::prelude::*;

/// Demo screen for the rich text editor.
#[component]
pub fn EditorDemo() -> Element {
    let mut content = use_signal(|| String::new());
    let mut simple_content = use_signal(|| String::new());
    let mut show_preview = use_signal(|| false);

    rsx! {
        div {
            class: "container mx-auto px-4 py-8 max-w-6xl",

            // Header
            div {
                class: "mb-8",
                h1 {
                    class: "text-3xl font-bold text-gray-900 dark:text-gray-100 mb-2",
                    "Rich Text Editor Demo"
                }
                p {
                    class: "text-gray-600 dark:text-gray-400",
                    "Full-featured WYSIWYG editor with AST-first architecture"
                }
            }

            // Full Editor Section
            section {
                class: "mb-12",
                h2 {
                    class: "text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "Full Editor"
                }
                p {
                    class: "text-gray-600 dark:text-gray-400 mb-4",
                    "Complete editor with all formatting options, media embeds, and more."
                }

                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden border border-gray-200 dark:border-gray-700",

                    RichTextEditor {
                        initial_value: None,
                        on_change: move |new_content| {
                            content.set(new_content);
                        },
                        placeholder: "Start writing your content here...".to_string(),
                        readonly: false,
                        class: "min-h-[400px]".to_string(),
                    }
                }

                // Preview toggle
                div {
                    class: "mt-4 flex items-center gap-4",
                    button {
                        class: "px-4 py-2 bg-blue-600 dark:bg-blue-700 text-white rounded-md hover:bg-blue-700 dark:hover:bg-blue-600",
                        onclick: move |_| show_preview.set(!show_preview()),
                        if *show_preview.read() {
                            "Hide Preview"
                        } else {
                            "Show Preview"
                        }
                    }

                    if !content.read().is_empty() {
                        span {
                            class: "text-sm text-gray-500 dark:text-gray-400",
                            "Content length: {content.read().len()} characters"
                        }
                    }
                }

                // Preview panel
                if *show_preview.read() && !content.read().is_empty() {
                    div {
                        class: "mt-6 p-6 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700",
                        h3 {
                            class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                            "Preview"
                        }
                        ContentViewer {
                            value: content.read().clone(),
                            class: "prose dark:prose-invert max-w-none".to_string(),
                        }

                        details {
                            class: "mt-4",
                            summary {
                                class: "cursor-pointer text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200",
                                "View JSON"
                            }
                            pre {
                                class: "mt-2 p-4 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 overflow-x-auto text-xs",
                                code {
                                    class: "text-gray-900 dark:text-gray-100",
                                    "{content}"
                                }
                            }
                        }
                    }
                }
            }

            // Simple Editor Section
            section {
                class: "mb-12",
                h2 {
                    class: "text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "Simple Editor"
                }
                p {
                    class: "text-gray-600 dark:text-gray-400 mb-4",
                    "Minimal editor variant with basic formatting options."
                }

                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden border border-gray-200 dark:border-gray-700",

                    SimpleEditor {
                        initial_value: None,
                        on_change: move |new_content| {
                            simple_content.set(new_content);
                        },
                        placeholder: "Quick note or comment...".to_string(),
                    }
                }
            }

            // Features Section
            section {
                class: "mb-12",
                h2 {
                    class: "text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "Features"
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-6",

                    FeatureCard {
                        title: "Text Formatting",
                        description: "Bold, italic, underline, strikethrough, inline code, and more.",
                        icon: "B",
                    }

                    FeatureCard {
                        title: "Headings & Blocks",
                        description: "Multiple heading levels, paragraphs, quotes, and code blocks.",
                        icon: "H",
                    }

                    FeatureCard {
                        title: "Lists",
                        description: "Bullet lists, numbered lists, and task lists with checkboxes.",
                        icon: "•",
                    }

                    FeatureCard {
                        title: "Links & Media",
                        description: "Insert links, images, and embed YouTube/X content.",
                        icon: "🔗",
                    }

                    FeatureCard {
                        title: "Security",
                        description: "Built-in HTML sanitization prevents XSS attacks.",
                        icon: "🔒",
                    }

                    FeatureCard {
                        title: "AST-First",
                        description: "Clean JSON representation for storage and API integration.",
                        icon: "📋",
                    }
                }
            }

            // Usage Examples Section
            section {
                class: "mb-12",
                h2 {
                    class: "text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "Usage Example"
                }

                div {
                    class: "bg-gray-900 dark:bg-gray-950 rounded-lg p-6 overflow-x-auto border border-gray-700",
                    pre {
                        class: "text-sm text-gray-100 dark:text-gray-200",
                        code {
                            r#"use crate::components::editor::RichTextEditor;

#[component]
fn MyComponent() -> Element {{
    let mut content = use_signal(|| String::new());

    rsx! {{
        RichTextEditor {{
            initial_value: None,
            on_change: move |new_content| {{
                // Save to database, update state, etc.
                content.set(new_content);
            }},
            placeholder: "Start writing...".to_string(),
        }}
    }}
}}"#
                        }
                    }
                }
            }

            // Keyboard Shortcuts
            section {
                class: "mb-12",
                h2 {
                    class: "text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4",
                    "Keyboard Shortcuts"
                }

                div {
                    class: "bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden border border-gray-200 dark:border-gray-700",
                    table {
                        class: "w-full",
                        thead {
                            class: "bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700",
                            tr {
                                th {
                                    class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "Shortcut"
                                }
                                th {
                                    class: "px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider",
                                    "Action"
                                }
                            }
                        }
                        tbody {
                            class: "bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700",
                            ShortcutRow { shortcut: "Ctrl+B", action: "Bold" }
                            ShortcutRow { shortcut: "Ctrl+I", action: "Italic" }
                            ShortcutRow { shortcut: "Ctrl+U", action: "Underline" }
                            ShortcutRow { shortcut: "Ctrl+K", action: "Insert Link" }
                            ShortcutRow { shortcut: "Enter", action: "New Paragraph" }
                            ShortcutRow { shortcut: "Shift+Enter", action: "Line Break" }
                            ShortcutRow { shortcut: "Ctrl+Z", action: "Undo" }
                            ShortcutRow { shortcut: "Ctrl+Y", action: "Redo" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureCard(title: String, description: String, icon: String) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-lg p-6 shadow-md hover:shadow-lg transition-shadow border border-gray-200 dark:border-gray-700",
            div {
                class: "text-3xl mb-3",
                "{icon}"
            }
            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2",
                "{title}"
            }
            p {
                class: "text-gray-600 dark:text-gray-400 text-sm",
                "{description}"
            }
        }
    }
}

#[component]
fn ShortcutRow(shortcut: String, action: String) -> Element {
    rsx! {
        tr {
            td {
                class: "px-6 py-4 whitespace-nowrap",
                code {
                    class: "px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-sm font-mono text-gray-800 dark:text-gray-200",
                    "{shortcut}"
                }
            }
            td {
                class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100",
                "{action}"
            }
        }
    }
}
