//! AST data model for the rich text editor.
//!
//! This module defines the core document structure as an Abstract Syntax Tree (AST)
//! that can be serialized to/from JSON and rendered to HTML.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root document structure containing all blocks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Doc {
    pub blocks: Vec<Block>,
}

impl Default for Doc {
    fn default() -> Self {
        Self {
            blocks: vec![Block::new_paragraph()],
        }
    }
}

/// A block-level element in the document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub id: String,
    pub kind: BlockKind,
    #[serde(default)]
    pub align: BlockAlign,
    #[serde(default)]
    pub attrs: HashMap<String, String>,
    pub children: Vec<Inline>,
}

impl Block {
    pub fn new(kind: BlockKind) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            align: BlockAlign::default(),
            attrs: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn new_paragraph() -> Self {
        Self::new(BlockKind::Paragraph)
    }

    pub fn new_heading(level: u8) -> Self {
        Self::new(BlockKind::Heading { level })
    }

    pub fn new_table(rows: usize, cols: usize) -> Self {
        let headers = vec![vec![Inline::text("")]; cols];
        let rows = vec![vec![vec![Inline::text("")]; cols]; rows];
        let column_align = vec![TableAlign::default(); cols];
        Self::new(BlockKind::Table {
            headers,
            rows,
            column_align,
        })
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.children.push(Inline::text(text));
        self
    }

    pub fn with_children(mut self, children: Vec<Inline>) -> Self {
        self.children = children;
        self
    }
}

/// Block-level element types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BlockKind {
    Paragraph,
    Heading {
        level: u8,
    },
    BulletList {
        items: Vec<Block>,
    },
    OrderedList {
        items: Vec<Block>,
    },
    TaskList {
        items: Vec<Block>,
    },
    ListItem,
    TaskItem {
        checked: bool,
    },
    Quote,
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Image {
        src: String,
        alt: Option<String>,
        title: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        caption: Option<String>,
    },
    Embed {
        provider: EmbedProvider,
        url: String,
        title: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Rule,
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        #[serde(default)]
        column_align: Vec<TableAlign>,
    },
}

/// Column alignment for table cells.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TableAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Supported embed providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmbedProvider {
    Youtube,
    X,
    Generic,
}

/// Block alignment options.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlockAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Inline-level elements (text runs, breaks).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Inline {
    Text {
        text: String,
        #[serde(default)]
        marks: MarkSet,
        #[serde(skip_serializing_if = "Option::is_none")]
        link: Option<Link>,
    },
    HardBreak,
}

impl Inline {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            marks: MarkSet::default(),
            link: None,
        }
    }

    pub fn text_with_marks(text: impl Into<String>, marks: MarkSet) -> Self {
        Self::Text {
            text: text.into(),
            marks,
            link: None,
        }
    }

    pub fn link(text: impl Into<String>, href: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            marks: MarkSet::default(),
            link: Some(Link {
                href: href.into(),
                title: None,
                target_blank: false,
            }),
        }
    }
}

/// Text formatting marks (inline styles).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MarkSet {
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub strike: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub code: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<TextSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,
}

impl MarkSet {
    pub fn bold() -> Self {
        Self {
            bold: true,
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: true,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.bold
            && !self.italic
            && !self.underline
            && !self.strike
            && !self.code
            && self.size.is_none()
            && self.highlight.is_none()
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Text size variants.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Small,
    Normal,
    Lead,
}

/// Link metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub target_blank: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_default_doc() {
        let doc = Doc::default();
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(doc.blocks[0].kind, BlockKind::Paragraph));
    }

    #[test]
    fn it_serializes_paragraph() {
        let block = Block::new_paragraph().with_text("Hello world");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("paragraph"));
        assert!(json.contains("Hello world"));
    }

    #[test]
    fn it_deserializes_heading() {
        let json = r#"{"id":"test","kind":{"type":"heading","level":2},"align":"left","attrs":{},"children":[]}"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert!(matches!(block.kind, BlockKind::Heading { level: 2 }));
    }

    #[test]
    fn it_handles_marks() {
        let marks = MarkSet::bold();
        assert!(marks.bold);
        assert!(!marks.italic);
        assert!(!marks.is_empty());

        let empty = MarkSet::default();
        assert!(empty.is_empty());
    }
}
