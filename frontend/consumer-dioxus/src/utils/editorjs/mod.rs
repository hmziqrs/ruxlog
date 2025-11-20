use dioxus::prelude::*;
use ruxlog_shared::store::{EditorJsBlock, PostContent};

// ============================================================================
// EditorJS Block Renderers
// ============================================================================

fn render_header_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Header { data, .. } = block {
        let level = data.level;
        let text = data.text.clone();

        match level {
            1 => rsx! { h1 { class: "text-4xl font-bold mb-6", "{text}" } },
            2 => rsx! { h2 { class: "text-3xl font-bold mb-5", "{text}" } },
            3 => rsx! { h3 { class: "text-2xl font-bold mb-4", "{text}" } },
            4 => rsx! { h4 { class: "text-xl font-bold mb-3", "{text}" } },
            5 => rsx! { h5 { class: "text-lg font-bold mb-2", "{text}" } },
            6 => rsx! { h6 { class: "text-base font-bold mb-2", "{text}" } },
            _ => rsx! { h1 { class: "text-4xl font-bold mb-6", "{text}" } },
        }
    } else {
        rsx! {}
    }
}

fn render_paragraph_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Paragraph { data, .. } = block {
        let text = data
            .text
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");

        rsx! {
            p { class: "mb-4 leading-7 text-foreground/90", dangerous_inner_html: "{text}" }
        }
    } else {
        rsx! {}
    }
}

fn render_code_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Code { data, .. } = block {
        let code = data.code.clone();
        rsx! {
            div { class: "my-6 rounded-lg overflow-hidden border bg-muted/50",
                pre { class: "p-4 overflow-x-auto text-sm",
                    code { class: "font-mono", "{code}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_quote_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Quote { data, .. } = &block {
        let alignment = match data.alignment.as_str() {
            "center" => "text-center",
            "right" => "text-right",
            _ => "text-left",
        };
        let text = data.text.clone();
        let caption = data.caption.clone();

        rsx! {
            blockquote { class: "my-6 pl-6 border-l-4 border-primary/30 italic text-lg",
                p { class: "mb-2 {alignment}", "{text}" }
                if let Some(caption) = caption {
                    footer { class: "text-sm text-muted-foreground not-italic {alignment}", "— {caption}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_list_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::List { data, .. } = block {
        let list_items = data.items.clone();
        let is_ordered = data.style == "ordered";

        if is_ordered {
            rsx! {
                ol { class: "my-6 ml-6 list-decimal space-y-2",
                    for item in list_items {
                        li { class: "leading-7 pl-2", dangerous_inner_html: "{item}" }
                    }
                }
            }
        } else {
            rsx! {
                ul { class: "my-6 ml-6 list-disc space-y-2",
                    for item in list_items {
                        li { class: "leading-7 pl-2", dangerous_inner_html: "{item}" }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_image_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Image { data, .. } = block {
        let url = data.file.url.clone();
        let caption = &data.caption;

        rsx! {
            figure { class: "my-8",
                img {
                    src: "{url}",
                    alt: caption.as_deref().unwrap_or(""),
                    class: "w-full h-auto rounded-lg shadow-md"
                }
                if let Some(ref caption) = data.caption {
                    figcaption { class: "mt-3 text-sm text-center text-muted-foreground italic", "{caption}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_delimiter_block(_block: &EditorJsBlock) -> Element {
    rsx! {
        div { class: "my-8 flex items-center justify-center",
            div { class: "flex gap-2",
                span { class: "w-1 h-1 rounded-full bg-muted-foreground" }
                span { class: "w-1 h-1 rounded-full bg-muted-foreground" }
                span { class: "w-1 h-1 rounded-full bg-muted-foreground" }
            }
        }
    }
}

pub fn render_editorjs_content(content: &PostContent) -> Element {
    rsx! {
        div { class: "prose prose-neutral dark:prose-invert max-w-none",
            for block in &content.blocks {
                match block {
                    EditorJsBlock::Header { .. } => render_header_block(block),
                    EditorJsBlock::Paragraph { .. } => render_paragraph_block(block),
                    EditorJsBlock::List { .. } => render_list_block(block),
                    EditorJsBlock::Delimiter { .. } => render_delimiter_block(block),
                    EditorJsBlock::Image { .. } => render_image_block(block),
                    EditorJsBlock::Code { .. } => render_code_block(block),
                    EditorJsBlock::Quote { .. } => render_quote_block(block),
                    _ => rsx! {},
                }
            }
        }
    }
}
