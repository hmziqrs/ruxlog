//! HTML renderer for the rich text editor AST.
//!
//! This module converts the AST representation into safe, sanitized HTML
//! for display and storage.

use super::ast::*;
use super::sanitizer::{is_safe_color, sanitize_url};

/// Renders a document to HTML.
pub fn render_doc(doc: &Doc) -> String {
    let mut html = String::new();
    for block in &doc.blocks {
        html.push_str(&render_block(block));
    }
    html
}

/// Renders a single block to HTML.
pub fn render_block(block: &Block) -> String {
    let align_class = match block.align {
        BlockAlign::Left => "",
        BlockAlign::Center => " text-center",
        BlockAlign::Right => " text-right",
        BlockAlign::Justify => " text-justify",
    };

    match &block.kind {
        BlockKind::Paragraph => {
            let content = render_inlines(&block.children);
            format!(
                "<p class=\"mb-4 text-gray-900 dark:text-gray-100{}\">{}</p>\n",
                align_class, content
            )
        }
        BlockKind::Heading { level } => {
            let content = render_inlines(&block.children);
            let size_class = match level {
                1 => " text-4xl font-bold",
                2 => " text-3xl font-bold",
                3 => " text-2xl font-semibold",
                4 => " text-xl font-semibold",
                5 => " text-lg font-medium",
                _ => " text-base font-medium",
            };
            format!(
                "<h{} class=\"mb-4 text-gray-900 dark:text-gray-100{}{}\">{}</h{}>\n",
                level, size_class, align_class, content, level
            )
        }
        BlockKind::BulletList { items } => {
            let mut html = format!(
                "<ul class=\"list-disc list-inside mb-4 text-gray-900 dark:text-gray-100{}\">\n",
                align_class
            );
            for item in items {
                html.push_str(&render_block(item));
            }
            html.push_str("</ul>\n");
            html
        }
        BlockKind::OrderedList { items } => {
            let mut html = format!(
                "<ol class=\"list-decimal list-inside mb-4 text-gray-900 dark:text-gray-100{}\">\n",
                align_class
            );
            for item in items {
                html.push_str(&render_block(item));
            }
            html.push_str("</ol>\n");
            html
        }
        BlockKind::TaskList { items } => {
            let mut html = format!(
                "<ul class=\"space-y-2 mb-4 text-gray-900 dark:text-gray-100{}\">\n",
                align_class
            );
            for item in items {
                html.push_str(&render_block(item));
            }
            html.push_str("</ul>\n");
            html
        }
        BlockKind::ListItem => {
            let content = render_inlines(&block.children);
            format!("  <li class=\"ml-4\">{}</li>\n", content)
        }
        BlockKind::TaskItem { checked } => {
            let content = render_inlines(&block.children);
            let checkbox = if *checked {
                r#"<input type="checkbox" checked disabled class="mr-2">"#
            } else {
                r#"<input type="checkbox" disabled class="mr-2">"#
            };
            format!(
                "  <li class=\"flex items-start\">{}{}</li>\n",
                checkbox, content
            )
        }
        BlockKind::Quote => {
            let content = render_inlines(&block.children);
            format!(
                "<blockquote class=\"border-l-4 border-gray-300 dark:border-gray-600 pl-4 italic mb-4 text-gray-700 dark:text-gray-300{}\">{}</blockquote>\n",
                align_class, content
            )
        }
        BlockKind::CodeBlock { language, code } => {
            let lang_class = language
                .as_ref()
                .map(|l| format!(" language-{}", html_escape(l)))
                .unwrap_or_default();
            format!(
                "<pre class=\"bg-gray-100 dark:bg-gray-800 rounded p-4 mb-4 overflow-x-auto border border-gray-200 dark:border-gray-700{}\"><code class=\"text-gray-900 dark:text-gray-100{}\">{}</code></pre>\n",
                align_class,
                lang_class,
                html_escape(code)
            )
        }
        BlockKind::Image {
            src,
            alt,
            title,
            width,
            height,
            caption,
        } => {
            let safe_src = sanitize_url(src).unwrap_or_else(|| "".to_string());
            let alt_attr = alt
                .as_ref()
                .map(|a| format!(" alt=\"{}\"", html_escape(a)))
                .unwrap_or_default();
            let title_attr = title
                .as_ref()
                .map(|t| format!(" title=\"{}\"", html_escape(t)))
                .unwrap_or_default();
            let width_attr = width
                .map(|w| format!(" width=\"{}\"", w))
                .unwrap_or_default();
            let height_attr = height
                .map(|h| format!(" height=\"{}\"", h))
                .unwrap_or_default();

            let mut html = format!("<figure class=\"mb-4{}\">\n", align_class);
            html.push_str(&format!(
                "  <img src=\"{}\"{}{}{}{}class=\"max-w-full h-auto\">\n",
                safe_src, alt_attr, title_attr, width_attr, height_attr
            ));
            if let Some(cap) = caption {
                html.push_str(&format!(
                    "  <figcaption class=\"text-sm text-gray-600 dark:text-gray-400 mt-2\">{}</figcaption>\n",
                    html_escape(cap)
                ));
            }
            html.push_str("</figure>\n");
            html
        }
        BlockKind::Embed {
            provider,
            url,
            title,
            width,
            height,
        } => {
            let safe_url = sanitize_url(url).unwrap_or_else(|| "".to_string());
            let w = width.unwrap_or(560);
            let h = height.unwrap_or(315);

            let embed_html = match provider {
                EmbedProvider::Youtube => {
                    // Extract YouTube video ID
                    let video_id = extract_youtube_id(&safe_url);
                    if let Some(id) = video_id {
                        format!(
                            r#"<iframe width="{}" height="{}" src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe>"#,
                            w, h, id
                        )
                    } else {
                        format!("<p>Invalid YouTube URL</p>")
                    }
                }
                EmbedProvider::X => {
                    // For X/Twitter, we'd typically use their embed API
                    format!(
                        r#"<blockquote class="twitter-tweet"><a href="{}">View on X</a></blockquote>"#,
                        safe_url
                    )
                }
                EmbedProvider::Generic => {
                    format!(
                        r#"<iframe src="{}" width="{}" height="{}" frameborder="0"></iframe>"#,
                        safe_url, w, h
                    )
                }
            };

            let mut html = format!("<div class=\"embed-container mb-4{}\">\n", align_class);
            html.push_str(&format!("  {}\n", embed_html));
            if let Some(t) = title {
                html.push_str(&format!(
                    "  <p class=\"text-sm text-gray-600 dark:text-gray-400 mt-2\">{}</p>\n",
                    html_escape(t)
                ));
            }
            html.push_str("</div>\n");
            html
        }
        BlockKind::Rule => {
            format!("<hr class=\"my-8 border-gray-300\">\n")
        }
        BlockKind::Table {
            headers,
            rows,
            column_align,
        } => {
            let mut html = format!(
                "<div class=\"mb-4 overflow-x-auto{}\"><table class=\"min-w-full border-collapse border border-gray-300 dark:border-gray-600\">\n",
                align_class
            );

            // Render table header
            if !headers.is_empty() {
                html.push_str("  <thead class=\"bg-gray-100 dark:bg-gray-700\">\n    <tr>\n");
                for (col_idx, header_cell) in headers.iter().enumerate() {
                    let align = column_align.get(col_idx).unwrap_or(&TableAlign::Left);
                    let align_class = match align {
                        TableAlign::Left => " text-left",
                        TableAlign::Center => " text-center",
                        TableAlign::Right => " text-right",
                    };
                    let content = render_inlines(header_cell);
                    html.push_str(&format!(
                        "      <th class=\"border border-gray-300 dark:border-gray-600 px-4 py-2 font-semibold text-gray-900 dark:text-gray-100{}\">{}</th>\n",
                        align_class, content
                    ));
                }
                html.push_str("    </tr>\n  </thead>\n");
            }

            // Render table rows
            if !rows.is_empty() {
                html.push_str("  <tbody>\n");
                for row in rows {
                    html.push_str("    <tr>\n");
                    for (col_idx, cell) in row.iter().enumerate() {
                        let align = column_align.get(col_idx).unwrap_or(&TableAlign::Left);
                        let align_class = match align {
                            TableAlign::Left => " text-left",
                            TableAlign::Center => " text-center",
                            TableAlign::Right => " text-right",
                        };
                        let content = render_inlines(cell);
                        html.push_str(&format!(
                            "      <td class=\"border border-gray-300 dark:border-gray-600 px-4 py-2 text-gray-900 dark:text-gray-100{}\">{}</td>\n",
                            align_class, content
                        ));
                    }
                    html.push_str("    </tr>\n");
                }
                html.push_str("  </tbody>\n");
            }

            html.push_str("</table></div>\n");
            html
        }
    }
}

/// Renders a list of inline elements to HTML.
pub fn render_inlines(inlines: &[Inline]) -> String {
    let mut html = String::new();
    for inline in inlines {
        html.push_str(&render_inline(inline));
    }
    html
}

/// Renders a single inline element to HTML.
pub fn render_inline(inline: &Inline) -> String {
    match inline {
        Inline::Text { text, marks, link } => {
            let mut content = html_escape(text);

            // Apply marks from innermost to outermost
            if marks.code {
                content = format!(
                    "<code class=\"bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-100 px-1 rounded\">{}</code>",
                    content
                );
            }

            if let Some(highlight) = &marks.highlight {
                if is_safe_color(highlight) {
                    content = format!(
                        "<mark style=\"background-color: {}\">{}</mark>",
                        html_escape(highlight),
                        content
                    );
                } else {
                    content = format!("<mark>{}</mark>", content);
                }
            }

            if marks.strike {
                content = format!("<s>{}</s>", content);
            }

            if marks.underline {
                content = format!("<u>{}</u>", content);
            }

            if marks.italic {
                content = format!("<em>{}</em>", content);
            }

            if marks.bold {
                content = format!("<strong>{}</strong>", content);
            }

            // Apply text size
            if let Some(size) = marks.size {
                let size_class = match size {
                    TextSize::Small => "text-sm",
                    TextSize::Normal => "text-base",
                    TextSize::Lead => "text-lg",
                };
                content = format!("<span class=\"{}\">{}</span>", size_class, content);
            }

            // Apply link wrapper (outermost)
            if let Some(link_data) = link {
                if let Some(safe_href) = sanitize_url(&link_data.href) {
                    let title_attr = link_data
                        .title
                        .as_ref()
                        .map(|t| format!(" title=\"{}\"", html_escape(t)))
                        .unwrap_or_default();
                    let target_attr = if link_data.target_blank {
                        r#" target="_blank" rel="noopener noreferrer""#
                    } else {
                        ""
                    };
                    content = format!(
                        "<a href=\"{}\"{}{}class=\"text-blue-600 hover:underline\">{}</a>",
                        safe_href, title_attr, target_attr, content
                    );
                }
            }

            content
        }
        Inline::HardBreak => "<br>\n".to_string(),
    }
}

/// Escapes HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Extracts YouTube video ID from various YouTube URL formats.
fn extract_youtube_id(url: &str) -> Option<String> {
    let url_lower = url.to_lowercase();

    // youtu.be/VIDEO_ID
    if let Some(idx) = url_lower.find("youtu.be/") {
        let start = idx + 9;
        let id = url[start..]
            .split(&['?', '&'][..])
            .next()
            .unwrap_or("")
            .to_string();
        if !id.is_empty() {
            return Some(id);
        }
    }

    // youtube.com/watch?v=VIDEO_ID
    if url_lower.contains("youtube.com/watch") {
        if let Some(v_idx) = url.find("v=") {
            let start = v_idx + 2;
            let id = url[start..]
                .split(&['&', '#'][..])
                .next()
                .unwrap_or("")
                .to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }
    }

    // youtube.com/embed/VIDEO_ID
    if let Some(idx) = url_lower.find("youtube.com/embed/") {
        let start = idx + 18;
        let id = url[start..]
            .split(&['?', '&'][..])
            .next()
            .unwrap_or("")
            .to_string();
        if !id.is_empty() {
            return Some(id);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_paragraph() {
        let block = Block::new_paragraph().with_text("Hello world");
        let html = render_block(&block);
        assert!(html.contains("<p"));
        assert!(html.contains("Hello world"));
        assert!(html.contains("</p>"));
    }

    #[test]
    fn it_renders_heading() {
        let block = Block::new_heading(2).with_text("Title");
        let html = render_block(&block);
        assert!(html.contains("<h2"));
        assert!(html.contains("Title"));
        assert!(html.contains("</h2>"));
    }

    #[test]
    fn it_renders_bold_text() {
        let inline = Inline::text_with_marks("Bold", MarkSet::bold());
        let html = render_inline(&inline);
        assert!(html.contains("<strong>"));
        assert!(html.contains("Bold"));
    }

    #[test]
    fn it_renders_link() {
        let inline = Inline::link("Click here", "https://example.com");
        let html = render_inline(&inline);
        assert!(html.contains("<a"));
        assert!(html.contains("https://example.com"));
        assert!(html.contains("Click here"));
    }

    #[test]
    fn it_escapes_html() {
        let escaped = html_escape("<script>alert('xss')</script>");
        assert!(!escaped.contains("<script"));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn it_extracts_youtube_id_from_watch_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        assert_eq!(extract_youtube_id(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn it_extracts_youtube_id_from_short_url() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        assert_eq!(extract_youtube_id(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn it_renders_code_block() {
        let block = Block::new(BlockKind::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn main() {}".to_string(),
        });
        let html = render_block(&block);
        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));
        assert!(html.contains("language-rust"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn it_renders_blockquote() {
        let block = Block::new(BlockKind::Quote).with_text("Quote text");
        let html = render_block(&block);
        assert!(html.contains("<blockquote"));
        assert!(html.contains("Quote text"));
    }

    #[test]
    fn it_renders_horizontal_rule() {
        let block = Block::new(BlockKind::Rule);
        let html = render_block(&block);
        assert!(html.contains("<hr"));
    }
}
