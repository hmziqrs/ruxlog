//! HTML to AST parser for the rich text editor.
//!
//! Converts sanitized HTML back into our internal AST representation
//! for structured content persistence and editing.

use super::ast::*;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Parses HTML content into an AST Document.
///
/// This parser expects sanitized HTML that matches our editor's output format.
/// It handles all block types (paragraphs, headings, lists, images, embeds, etc.)
/// and inline formatting (bold, italic, links, etc.).
///
/// # Example
///
/// ```ignore
/// let html = r#"<p>Hello <strong>world</strong>!</p>"#;
/// let doc = parse_html(html);
/// assert_eq!(doc.blocks.len(), 1);
/// ```
pub fn parse_html(html: &str) -> Doc {
    let fragment = Html::parse_fragment(html);
    let mut blocks = Vec::new();
    let mut block_id_counter = 0;

    // Try common block-level selectors
    let block_selectors = vec![
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "ul",
        "ol",
        "blockquote",
        "pre",
        "hr",
        "table",
        "figure",
        "img:not(figure img)", // Only standalone images, not those inside figures
    ];

    for tag in block_selectors {
        if let Ok(selector) = Selector::parse(tag) {
            for element in fragment.select(&selector) {
                if let Some(block) = parse_element_to_block(element, &mut block_id_counter) {
                    blocks.push(block);
                }
            }
        }
    }

    // Ensure at least one empty paragraph
    if blocks.is_empty() {
        blocks.push(Block {
            id: generate_id(&mut block_id_counter),
            kind: BlockKind::Paragraph,
            align: BlockAlign::Left,
            attrs: HashMap::new(),
            children: vec![],
        });
    }

    Doc { blocks }
}

/// Generates a unique block ID.
fn generate_id(counter: &mut usize) -> String {
    *counter += 1;
    format!("block-{}", counter)
}

/// Parses an element to a Block.
fn parse_element_to_block(
    el: scraper::element_ref::ElementRef,
    counter: &mut usize,
) -> Option<Block> {
    let tag = el.value().name();
    let align = parse_alignment(el);
    let attrs = parse_attributes(el);
    let id = generate_id(counter);

    let kind = match tag {
        "p" => BlockKind::Paragraph,
        "h1" => BlockKind::Heading { level: 1 },
        "h2" => BlockKind::Heading { level: 2 },
        "h3" => BlockKind::Heading { level: 3 },
        "h4" => BlockKind::Heading { level: 4 },
        "h5" => BlockKind::Heading { level: 5 },
        "h6" => BlockKind::Heading { level: 6 },
        "ul" => {
            if has_class(el, "task-list") {
                let items = parse_task_list_items(el, counter);
                BlockKind::TaskList { items }
            } else {
                let items = parse_list_items(el, counter);
                BlockKind::BulletList { items }
            }
        }
        "ol" => {
            let items = parse_list_items(el, counter);
            BlockKind::OrderedList { items }
        }
        "blockquote" => BlockKind::Quote,
        "pre" => {
            let (language, code) = parse_code_block(el);
            BlockKind::CodeBlock { language, code }
        }
        "hr" => BlockKind::Rule,
        "table" => {
            return parse_table_block(el, counter);
        }
        "figure" => {
            // Check if this is an image or embed
            if let Some(img) = find_descendant(el, "img") {
                return parse_image_block(img, Some(el), counter);
            } else if let Some(iframe) = find_descendant(el, "iframe") {
                return parse_embed_block(iframe, counter);
            }
            return None;
        }
        "img" => {
            // Standalone image (not wrapped in figure)
            return parse_image_block(el, None, counter);
        }
        _ => return None,
    };

    let children = if matches!(
        kind,
        BlockKind::Paragraph | BlockKind::Heading { .. } | BlockKind::Quote
    ) {
        parse_inline_content(el)
    } else {
        vec![]
    };

    Some(Block {
        id,
        kind,
        align,
        attrs,
        children,
    })
}

/// Parses inline content from an element.
fn parse_inline_content(el: scraper::element_ref::ElementRef) -> Vec<Inline> {
    let mut inlines = Vec::new();
    parse_inline_html(&el.inner_html(), &mut inlines);

    if inlines.is_empty() {
        inlines.push(Inline::Text {
            text: String::new(),
            marks: MarkSet::default(),
            link: None,
        });
    }

    inlines
}

/// Parses inline HTML into Inline nodes.
fn parse_inline_html(html: &str, inlines: &mut Vec<Inline>) {
    let fragment = Html::parse_fragment(html);
    let marks = MarkSet::default();
    parse_inline_recursive(&fragment.root_element().inner_html(), inlines, &marks, None);
}

/// Recursively parses inline content from HTML string.
fn parse_inline_recursive(
    html: &str,
    inlines: &mut Vec<Inline>,
    marks: &MarkSet,
    link: Option<Link>,
) {
    let fragment = Html::parse_fragment(html);

    for node in fragment.root_element().children() {
        if let scraper::node::Node::Text(text) = node.value() {
            let text_str = text.to_string();
            if !text_str.is_empty() {
                inlines.push(Inline::Text {
                    text: text_str,
                    marks: marks.clone(),
                    link: link.clone(),
                });
            }
        } else if let Some(element) = scraper::ElementRef::wrap(node) {
            let tag = element.value().name();
            let mut new_marks = marks.clone();
            let mut new_link = link.clone();

            match tag {
                "strong" | "b" => new_marks.bold = true,
                "em" | "i" => new_marks.italic = true,
                "u" => new_marks.underline = true,
                "s" | "strike" | "del" => new_marks.strike = true,
                "code" => new_marks.code = true,
                "mark" => {
                    new_marks.highlight = element
                        .value()
                        .attr("data-color")
                        .and_then(|c| c.parse().ok());
                }
                "small" => new_marks.size = Some(TextSize::Small),
                "span" => {
                    if has_class(element, "text-lg") || has_class(element, "lead") {
                        new_marks.size = Some(TextSize::Lead);
                    }
                }
                "a" => {
                    if let Some(href) = element.value().attr("href") {
                        new_link = Some(Link {
                            href: href.to_string(),
                            title: element.value().attr("title").map(|s| s.to_string()),
                            target_blank: element.value().attr("target") == Some("_blank"),
                        });
                    }
                }
                "br" => {
                    inlines.push(Inline::HardBreak);
                    continue;
                }
                _ => {}
            }

            // Recurse with updated marks
            parse_inline_recursive(&element.inner_html(), inlines, &new_marks, new_link);
        }
    }
}

/// Parses list items (for bullet and ordered lists).
fn parse_list_items(el: scraper::element_ref::ElementRef, counter: &mut usize) -> Vec<Block> {
    let mut items = Vec::new();

    if let Ok(selector) = Selector::parse("li") {
        for li in el.select(&selector) {
            let children = parse_inline_content(li);
            items.push(Block {
                id: generate_id(counter),
                kind: BlockKind::ListItem,
                align: BlockAlign::Left,
                attrs: HashMap::new(),
                children,
            });
        }
    }

    items
}

/// Parses task list items.
fn parse_task_list_items(el: scraper::element_ref::ElementRef, counter: &mut usize) -> Vec<Block> {
    let mut items = Vec::new();

    if let Ok(selector) = Selector::parse("li") {
        for li in el.select(&selector) {
            // Check for checkbox
            let checked = if let Ok(input_sel) = Selector::parse("input[type='checkbox']") {
                li.select(&input_sel)
                    .next()
                    .and_then(|input| Some(input.value().attr("checked").is_some()))
                    .unwrap_or(false)
            } else {
                false
            };

            // Get text content (excluding checkbox)
            let text = li.text().collect::<String>().trim().to_string();
            let children = if text.is_empty() {
                vec![]
            } else {
                vec![Inline::Text {
                    text,
                    marks: MarkSet::default(),
                    link: None,
                }]
            };

            items.push(Block {
                id: generate_id(counter),
                kind: BlockKind::TaskItem { checked },
                align: BlockAlign::Left,
                attrs: HashMap::new(),
                children,
            });
        }
    }

    items
}

/// Parses a code block's language and content.
fn parse_code_block(pre_el: scraper::element_ref::ElementRef) -> (Option<String>, String) {
    // Look for <code> child
    if let Some(code_el) = find_descendant(pre_el, "code") {
        let language = code_el
            .value()
            .attr("class")
            .and_then(|cls| {
                cls.split_whitespace()
                    .find(|c| c.starts_with("language-"))
                    .map(|c| c.strip_prefix("language-").unwrap().to_string())
            })
            .or_else(|| code_el.value().attr("data-language").map(|s| s.to_string()));

        let code = code_el.text().collect::<String>();
        return (language, code);
    }

    // Fallback: extract text from pre directly
    let code = pre_el.text().collect::<String>();
    (None, code)
}

/// Parses an image block from a figure or img element.
fn parse_image_block(
    img_el: scraper::element_ref::ElementRef,
    figure_el: Option<scraper::element_ref::ElementRef>,
    counter: &mut usize,
) -> Option<Block> {
    let src = img_el.value().attr("src")?.to_string();
    let alt = img_el.value().attr("alt").map(|s| s.to_string());
    let title = img_el.value().attr("title").map(|s| s.to_string());
    let width = img_el
        .value()
        .attr("width")
        .and_then(|w| w.parse::<u32>().ok());
    let height = img_el
        .value()
        .attr("height")
        .and_then(|h| h.parse::<u32>().ok());

    // Check for figcaption
    let caption = figure_el
        .and_then(|fig| find_descendant(fig, "figcaption"))
        .map(|cap_el| cap_el.text().collect::<String>());

    Some(Block {
        id: generate_id(counter),
        kind: BlockKind::Image {
            src,
            alt,
            title,
            width,
            height,
            caption,
        },
        align: BlockAlign::Left,
        attrs: HashMap::new(),
        children: vec![],
    })
}

/// Parses an embed block from an iframe.
fn parse_embed_block(
    iframe_el: scraper::element_ref::ElementRef,
    counter: &mut usize,
) -> Option<Block> {
    let url = iframe_el.value().attr("src")?.to_string();
    let title = iframe_el.value().attr("title").map(|s| s.to_string());
    let width = iframe_el
        .value()
        .attr("width")
        .and_then(|w| w.parse::<u32>().ok());
    let height = iframe_el
        .value()
        .attr("height")
        .and_then(|h| h.parse::<u32>().ok());

    let provider = detect_embed_provider(&url);

    Some(Block {
        id: generate_id(counter),
        kind: BlockKind::Embed {
            provider,
            url,
            title,
            width,
            height,
        },
        align: BlockAlign::Left,
        attrs: HashMap::new(),
        children: vec![],
    })
}

/// Detects the embed provider from URL.
fn detect_embed_provider(url: &str) -> EmbedProvider {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        EmbedProvider::Youtube
    } else if url.contains("twitter.com") || url.contains("x.com") {
        EmbedProvider::X
    } else {
        EmbedProvider::Generic
    }
}

/// Parses a table element into a Table block.
fn parse_table_block(
    table_el: scraper::element_ref::ElementRef,
    counter: &mut usize,
) -> Option<Block> {
    let mut headers: Vec<Vec<Inline>> = Vec::new();
    let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();
    let mut column_align: Vec<TableAlign> = Vec::new();

    // Parse thead for headers
    if let Some(thead) = find_descendant(table_el, "thead") {
        if let Some(tr) = find_descendant(thead, "tr") {
            let th_selector = Selector::parse("th").ok()?;
            for (_col_idx, th) in tr.select(&th_selector).enumerate() {
                let cell_content = parse_inline_content(th);
                headers.push(cell_content);

                // Parse column alignment from th
                let align = parse_cell_alignment(th);
                column_align.push(align);
            }
        }
    }

    // Parse tbody for data rows
    if let Some(tbody) = find_descendant(table_el, "tbody") {
        let tr_selector = Selector::parse("tr").ok()?;
        for tr in tbody.select(&tr_selector) {
            let mut row_cells: Vec<Vec<Inline>> = Vec::new();
            let td_selector = Selector::parse("td").ok()?;
            for td in tr.select(&td_selector) {
                let cell_content = parse_inline_content(td);
                row_cells.push(cell_content);
            }
            if !row_cells.is_empty() {
                rows.push(row_cells);
            }
        }
    }

    // If no explicit thead/tbody, try to parse tr elements directly
    if headers.is_empty() && rows.is_empty() {
        let tr_selector = Selector::parse("tr").ok()?;
        let mut first_row = true;
        for tr in table_el.select(&tr_selector) {
            // First row might be headers if it contains th elements
            if first_row {
                let th_selector = Selector::parse("th").ok()?;
                let th_count = tr.select(&th_selector).count();

                if th_count > 0 {
                    // This row has th elements, treat as header
                    for th in tr.select(&th_selector) {
                        let cell_content = parse_inline_content(th);
                        headers.push(cell_content);
                        let align = parse_cell_alignment(th);
                        column_align.push(align);
                    }
                    first_row = false;
                    continue;
                }
            }

            // Regular data row with td elements
            let td_selector = Selector::parse("td").ok()?;
            let mut row_cells: Vec<Vec<Inline>> = Vec::new();
            for td in tr.select(&td_selector) {
                let cell_content = parse_inline_content(td);
                row_cells.push(cell_content);
            }
            if !row_cells.is_empty() {
                rows.push(row_cells);
            }
            first_row = false;
        }
    }

    // Ensure column_align has enough entries
    let max_cols = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    while column_align.len() < max_cols {
        column_align.push(TableAlign::default());
    }

    Some(Block {
        id: generate_id(counter),
        kind: BlockKind::Table {
            headers,
            rows,
            column_align,
        },
        align: BlockAlign::Left,
        attrs: HashMap::new(),
        children: vec![],
    })
}

/// Parses cell alignment from th/td element.
fn parse_cell_alignment(cell_el: scraper::element_ref::ElementRef) -> TableAlign {
    if let Some(class) = cell_el.value().attr("class") {
        if class.contains("text-center") {
            return TableAlign::Center;
        } else if class.contains("text-right") {
            return TableAlign::Right;
        }
    }

    if let Some(style) = cell_el.value().attr("style") {
        if style.contains("text-align: center") || style.contains("text-align:center") {
            return TableAlign::Center;
        } else if style.contains("text-align: right") || style.contains("text-align:right") {
            return TableAlign::Right;
        }
    }

    TableAlign::Left
}

/// Parses text alignment from element classes or styles.
fn parse_alignment(el: scraper::element_ref::ElementRef) -> BlockAlign {
    if let Some(class) = el.value().attr("class") {
        if class.contains("text-center") {
            return BlockAlign::Center;
        } else if class.contains("text-right") {
            return BlockAlign::Right;
        } else if class.contains("text-justify") {
            return BlockAlign::Justify;
        }
    }

    if let Some(style) = el.value().attr("style") {
        if style.contains("text-align: center") || style.contains("text-align:center") {
            return BlockAlign::Center;
        } else if style.contains("text-align: right") || style.contains("text-align:right") {
            return BlockAlign::Right;
        } else if style.contains("text-align: justify") || style.contains("text-align:justify") {
            return BlockAlign::Justify;
        }
    }

    BlockAlign::Left
}

/// Extracts custom attributes from an element.
fn parse_attributes(el: scraper::element_ref::ElementRef) -> HashMap<String, String> {
    let mut attrs = HashMap::new();

    for (name, value) in el.value().attrs() {
        // Store data-* attributes
        if name.starts_with("data-") {
            attrs.insert(name.to_string(), value.to_string());
        }
    }

    attrs
}

/// Checks if an element has a specific class.
fn has_class(el: scraper::element_ref::ElementRef, class_name: &str) -> bool {
    el.value()
        .attr("class")
        .map(|classes| classes.split_whitespace().any(|c| c == class_name))
        .unwrap_or(false)
}

/// Finds a descendant element with a specific tag name.
fn find_descendant<'a>(
    el: scraper::element_ref::ElementRef<'a>,
    tag_name: &str,
) -> Option<scraper::element_ref::ElementRef<'a>> {
    if let Ok(selector) = Selector::parse(tag_name) {
        el.select(&selector).next()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_empty_html() {
        let doc = parse_html("");
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(doc.blocks[0].kind, BlockKind::Paragraph));
    }

    #[test]
    fn it_parses_simple_paragraph() {
        let html = r#"<p>Hello world</p>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(doc.blocks[0].kind, BlockKind::Paragraph));
        assert!(!doc.blocks[0].children.is_empty());
    }

    #[test]
    fn it_parses_bold_text() {
        let html = r#"<p>Hello <strong>world</strong>!</p>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        let children = &doc.blocks[0].children;

        // Should have text with bold marks
        let has_bold = children.iter().any(|inline| {
            if let Inline::Text { marks, .. } = inline {
                marks.bold
            } else {
                false
            }
        });
        assert!(has_bold);
    }

    #[test]
    fn it_parses_heading() {
        let html = r#"<h2>Title</h2>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(
            doc.blocks[0].kind,
            BlockKind::Heading { level: 2 }
        ));
    }

    #[test]
    fn it_parses_bullet_list() {
        let html = r#"<ul><li>Item 1</li><li>Item 2</li></ul>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        if let BlockKind::BulletList { items } = &doc.blocks[0].kind {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected BulletList");
        }
    }

    #[test]
    fn it_parses_link() {
        let html = r#"<p><a href="https://example.com" target="_blank">Link</a></p>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        let children = &doc.blocks[0].children;

        let has_link = children.iter().any(|inline| {
            if let Inline::Text { link, .. } = inline {
                link.is_some()
            } else {
                false
            }
        });
        assert!(has_link);
    }

    #[test]
    fn it_parses_code_block() {
        let html = r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        if let BlockKind::CodeBlock { language, code } = &doc.blocks[0].kind {
            assert_eq!(language.as_deref(), Some("rust"));
            assert!(code.contains("fn main()"));
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn it_parses_image() {
        let html = r#"<figure><img src="/image.jpg" alt="Test" width="800" height="600"><figcaption>Caption</figcaption></figure>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        if let BlockKind::Image {
            src,
            alt,
            width,
            height,
            caption,
            ..
        } = &doc.blocks[0].kind
        {
            assert_eq!(src, "/image.jpg");
            assert_eq!(alt.as_deref(), Some("Test"));
            assert_eq!(*width, Some(800));
            assert_eq!(*height, Some(600));
            assert_eq!(caption.as_deref(), Some("Caption"));
        } else {
            panic!("Expected Image block");
        }
    }

    #[test]
    fn it_parses_alignment() {
        let html = r#"<p class="text-center">Centered</p>"#;
        let doc = parse_html(html);
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(doc.blocks[0].align, BlockAlign::Center));
    }
}
