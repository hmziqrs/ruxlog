use dioxus::prelude::*;
use ruxlog_shared::store::{EditorJsBlock, PostContent};

// M-9 (defense-in-depth XSS): the server already strips dangerous markup from
// post content with ammonia on write, but every `dangerous_inner_html` sink is
// an XSS hole if ANY unsanitized string ever reaches the client (a buggy
// migration, a stale cache, a future code path that bypasses sanitization, a
// contributor post from before sanitization shipped). Sanitize again, on the
// client, immediately before injection. Ammonia's conservative default strips
// <script>, event-handler attributes, javascript: URLs, etc. This is the same
// library the server uses, so the two layers agree on what is safe.
fn sanitize_html(html: &str) -> String {
    ammonia::clean(html)
}

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
        // Decode entities, THEN sanitize — ammonia must see the real markup so
        // it can strip anything the server layer missed.
        let decoded = data
            .text
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        let text = sanitize_html(&decoded);

        rsx! {
            p { class: "mb-4 leading-7", dangerous_inner_html: "{text}" }
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
                    footer { class: "text-sm not-italic {alignment}", "— {caption}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_list_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::List { data, .. } = block {
        // EditorJS list items may carry inline markup (<b>, <a>, ...); sanitize
        // each before it hits the inner-HTML sink.
        let list_items: Vec<String> = data.items.iter().map(|item| sanitize_html(item)).collect();
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
                    figcaption { class: "mt-3 text-sm text-center italic", "{caption}" }
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

fn render_raw_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Raw { data, .. } = block {
        // The raw block is the highest-risk sink: it is arbitrary HTML by
        // definition. Sanitize unconditionally before injection.
        let html = sanitize_html(&data.html);
        rsx! {
            div { class: "my-6", dangerous_inner_html: "{html}" }
        }
    } else {
        rsx! {}
    }
}

pub fn render_editorjs_content(content: &PostContent) -> Element {
    rsx! {
        div { class: "max-w-none",
            for block in &content.blocks {
                match block {
                    EditorJsBlock::Header { .. } => render_header_block(block),
                    EditorJsBlock::Paragraph { .. } => render_paragraph_block(block),
                    EditorJsBlock::List { .. } => render_list_block(block),
                    EditorJsBlock::Delimiter { .. } => render_delimiter_block(block),
                    EditorJsBlock::Image { .. } => render_image_block(block),
                    EditorJsBlock::Code { .. } => render_code_block(block),
                    EditorJsBlock::Quote { .. } => render_quote_block(block),
                    EditorJsBlock::Raw { .. } => render_raw_block(block),
                    _ => rsx! {},
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruxlog_shared::store::{ListBlock, ParagraphBlock, RawBlock};

    #[test]
    fn raw_block_render_path_does_not_panic() {
        let content = PostContent {
            time: 0,
            version: "test".to_string(),
            blocks: vec![EditorJsBlock::Raw {
                id: Some("raw".to_string()),
                data: RawBlock {
                    html: "<h1>Hello</h1><p>world</p>".to_string(),
                },
            }],
        };

        let _ = render_editorjs_content(&content);
    }

    // M-9: the sanitize_html helper is the client-side XSS gate for every
    // dangerous_inner_html sink. Assert that each dangerous markup class the
    // server strips is also stripped here, so a stale/migrated/un-sanitized
    // payload can never execute in the browser.

    #[test]
    fn sanitize_strips_script_tags() {
        let cleaned = sanitize_html("<p>hi</p><script>alert(1)</script>");
        assert!(!cleaned.contains("<script>"));
        assert!(!cleaned.contains("alert"));
        assert!(cleaned.contains("<p>hi</p>"));
    }

    #[test]
    fn sanitize_strips_event_handler_attributes() {
        let cleaned = sanitize_html(r#"<img src="x" onerror="alert(1)">"#);
        assert!(!cleaned.contains("onerror"));
        assert!(!cleaned.contains("alert"));
    }

    #[test]
    fn sanitize_strips_javascript_urls() {
        let cleaned = sanitize_html(r#"<a href="javascript:alert(1)">x</a>"#);
        assert!(!cleaned.contains("javascript:"));
        assert!(!cleaned.contains("alert"));
    }

    #[test]
    fn sanitize_preserves_safe_inline_markup() {
        // Safe formatting tags survive so legit rich text still renders.
        let cleaned =
            sanitize_html("<b>bold</b> <i>italic</i> <a href=\"https://example.com\">link</a>");
        assert!(cleaned.contains("<b>bold</b>"));
        assert!(cleanly_contains_link(&cleaned));
    }

    fn cleanly_contains_link(cleaned: &str) -> bool {
        cleaned.contains("<a")
            && cleaned.contains("example.com")
            && !cleaned.contains("javascript:")
    }

    #[test]
    fn paragraph_block_sanitizes_script_payload() {
        // A paragraph whose decoded text carries a <script> must not pass it
        // through to the inner-HTML sink. We exercise the renderer indirectly
        // via the shared helper to avoid rendering a full Element in a unit
        // test; the renderer calls the same helper.
        let malicious = "<script>alert('xss')</script><p>ok</p>";
        assert!(!sanitize_html(malicious).contains("script"));
    }

    #[test]
    fn list_items_are_each_sanitized() {
        // Mirror render_list_block's per-item sanitization contract.
        let items = vec![
            "<b>one</b><script>x</script>".to_string(),
            "two<script>y</script>".to_string(),
        ];
        let cleaned: Vec<String> = items.iter().map(|i| sanitize_html(i)).collect();
        assert!(cleaned.iter().all(|c| !c.contains("<script>")));
        assert!(cleaned[0].contains("<b>one</b>"));
    }

    // Confirm the block data shapes compile against the renderers (catches
    // silent struct drift that would skip sanitization).
    #[test]
    fn list_and_paragraph_block_shapes_match_renderers() {
        let _list = ListBlock {
            style: "ordered".to_string(),
            items: vec!["<i>x</i>".to_string()],
        };
        let _para = ParagraphBlock {
            text: "plain".to_string(),
        };
    }
}
