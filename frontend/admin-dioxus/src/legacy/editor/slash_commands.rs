//! Slash commands component for quick block insertion.
//!
//! Displays a menu when user types '/' to insert different block types.

use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// Props for the SlashCommands component.
#[derive(Props, Clone, PartialEq)]
pub struct SlashCommandsProps {
    /// Whether the menu should be shown
    pub show: bool,
    /// Callback when a command is selected
    pub on_select: EventHandler<SlashCommand>,
    /// Callback when menu should be closed
    pub on_close: EventHandler<()>,
    /// ID of the editor element for positioning
    pub editor_id: String,
    /// Current search query (text after '/')
    #[props(default = String::new())]
    pub query: String,
}

/// Available slash commands.
#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    BulletList,
    OrderedList,
    TaskList,
    Quote,
    CodeBlock,
    Divider,
    Image,
    Table,
}

impl SlashCommand {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Paragraph,
            Self::Heading1,
            Self::Heading2,
            Self::Heading3,
            Self::BulletList,
            Self::OrderedList,
            Self::TaskList,
            Self::Quote,
            Self::CodeBlock,
            Self::Divider,
            Self::Image,
            Self::Table,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Paragraph => "Paragraph",
            Self::Heading1 => "Heading 1",
            Self::Heading2 => "Heading 2",
            Self::Heading3 => "Heading 3",
            Self::BulletList => "Bullet List",
            Self::OrderedList => "Numbered List",
            Self::TaskList => "Task List",
            Self::Quote => "Quote",
            Self::CodeBlock => "Code Block",
            Self::Divider => "Divider",
            Self::Image => "Image",
            Self::Table => "Table",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Paragraph => "¶",
            Self::Heading1 => "H1",
            Self::Heading2 => "H2",
            Self::Heading3 => "H3",
            Self::BulletList => "•",
            Self::OrderedList => "1.",
            Self::TaskList => "☑",
            Self::Quote => "❝",
            Self::CodeBlock => "</>",
            Self::Divider => "—",
            Self::Image => "🖼",
            Self::Table => "▦",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Paragraph => "Plain text paragraph",
            Self::Heading1 => "Large section heading",
            Self::Heading2 => "Medium section heading",
            Self::Heading3 => "Small section heading",
            Self::BulletList => "Unordered list",
            Self::OrderedList => "Numbered list",
            Self::TaskList => "Interactive checklist",
            Self::Quote => "Block quotation",
            Self::CodeBlock => "Code with syntax highlighting",
            Self::Divider => "Horizontal rule",
            Self::Image => "Insert image",
            Self::Table => "Insert table with rows and columns",
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let q = query.to_lowercase();
        self.label().to_lowercase().contains(&q) || self.description().to_lowercase().contains(&q)
    }
}

/// Slash commands menu component.
#[component]
pub fn SlashCommands(props: SlashCommandsProps) -> Element {
    let mut position = use_signal(|| MenuPosition {
        top: 0.0,
        left: 0.0,
    });
    let mut selected_index = use_signal(|| 0_usize);

    // Filter commands based on query
    let query_clone = props.query.clone();
    let filtered_commands = use_memo(move || {
        SlashCommand::all()
            .into_iter()
            .filter(|cmd| cmd.matches_query(&query_clone))
            .collect::<Vec<_>>()
    });

    // Update position when shown
    use_effect({
        let editor_id = props.editor_id.clone();
        let show = props.show;
        move || {
            if show {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Ok(Some(sel)) = window.get_selection() {
                            if sel.range_count() > 0 {
                                if let Ok(range) = sel.get_range_at(0) {
                                    let rect = range.get_bounding_client_rect();

                                    // Get editor element for bounds
                                    if let Some(editor) = document.get_element_by_id(&editor_id) {
                                        if let Ok(editor_html) = editor.dyn_into::<HtmlElement>() {
                                            let _editor_rect =
                                                editor_html.get_bounding_client_rect();

                                            // Position menu below cursor
                                            let left = rect.left();
                                            let top = rect.bottom() + 4.0; // Small gap

                                            position.set(MenuPosition { top, left });
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

    // Reset selected index when query changes
    use_effect(move || {
        let _q = props.query.clone();
        selected_index.set(0);
    });

    // Handle keyboard navigation
    let handle_key_down = move |evt: Event<KeyboardData>| {
        let key = evt.key();
        let commands = filtered_commands.read();

        match key {
            Key::ArrowDown => {
                evt.prevent_default();
                let current = selected_index();
                if current < commands.len().saturating_sub(1) {
                    selected_index.set(current + 1);
                }
            }
            Key::ArrowUp => {
                evt.prevent_default();
                if selected_index() > 0 {
                    selected_index.set(selected_index() - 1);
                }
            }
            Key::Enter => {
                evt.prevent_default();
                if let Some(cmd) = commands.get(selected_index()) {
                    props.on_select.call(cmd.clone());
                }
            }
            Key::Escape => {
                evt.prevent_default();
                props.on_close.call(());
            }
            _ => {}
        }
    };

    if !props.show {
        return rsx! {};
    }

    let pos = position();
    let commands = filtered_commands.read();

    if commands.is_empty() {
        return rsx! {
            div {
                class: "slash-commands-menu fixed z-50 bg-base-100 dark:bg-gray-800 border border-base-300 dark:border-gray-600 rounded-lg shadow-lg p-3 w-72",
                style: "top: {pos.top}px; left: {pos.left}px;",

                div { class: "text-sm text-base-content/60 dark:text-gray-400 text-center",
                    "No matching commands"
                }
            }
        };
    }

    rsx! {
        div {
            class: "slash-commands-menu fixed z-50 bg-base-100 dark:bg-gray-800 border border-base-300 dark:border-gray-600 rounded-lg shadow-lg py-2 w-80 max-h-96 overflow-y-auto",
            style: "top: {pos.top}px; left: {pos.left}px;",
            tabindex: "-1",
            onkeydown: handle_key_down,

            for (index, command) in commands.iter().enumerate() {
                {
                    let cmd = command.clone();
                    let is_selected = index == selected_index();
                    rsx! {
                        button {
                            key: "{index}",
                            class: if is_selected {
                                "w-full flex items-start gap-3 px-3 py-2 bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
                            } else {
                                "w-full flex items-start gap-3 px-3 py-2 hover:bg-base-200 dark:hover:bg-gray-700 transition-colors"
                            },
                            r#type: "button",
                            onclick: move |_| {
                                props.on_select.call(cmd.clone());
                            },
                            onmouseenter: move |_| {
                                selected_index.set(index);
                            },

                            // Icon
                            div { class: "flex-shrink-0 w-8 h-8 rounded bg-base-200 dark:bg-gray-700 flex items-center justify-center text-sm font-semibold",
                                "{command.icon()}"
                            }

                            // Label and description
                            div { class: "flex-1 text-left",
                                div { class: "text-sm font-medium text-base-content dark:text-gray-100",
                                    "{command.label()}"
                                }
                                div { class: "text-xs text-base-content/60 dark:text-gray-400 mt-0.5",
                                    "{command.description()}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Position for the slash commands menu.
#[derive(Clone, Copy, PartialEq)]
struct MenuPosition {
    top: f64,
    left: f64,
}
