# Rich Text Editor for Dioxus

A full-featured WYSIWYG rich text editor component built with an AST-first architecture for Dioxus applications.

## Features

- **Text Formatting**: Bold, italic, underline, strikethrough, inline code, text sizing, and highlighting
- **Block Types**: Paragraphs, headings (H1-H6), blockquotes, code blocks, horizontal rules
- **Lists**: Bullet lists, numbered lists, and task lists with checkboxes
- **Links & Media**: Insert hyperlinks, images with captions, and embed YouTube/X content
- **Security**: Built-in HTML sanitization prevents XSS attacks using `ammonia`
- **AST-First**: Clean JSON representation for easy storage and API integration
- **Accessibility**: Keyboard shortcuts and semantic HTML output
- **Customizable**: Toolbar can be customized or hidden for read-only views

## Quick Start

```rust
use crate::components::editor::RichTextEditor;

#[component]
fn MyComponent() -> Element {
    let mut content = use_signal(|| String::new());

    rsx! {
        RichTextEditor {
            initial_value: None,
            on_change: move |new_content| {
                content.set(new_content);
                // Save to database, update state, etc.
            },
            placeholder: "Start writing...".to_string(),
        }
    }
}
```

## Components

### RichTextEditor

The main full-featured editor component.

**Props:**
- `initial_value: Option<String>` - Initial document content as JSON
- `on_change: EventHandler<String>` - Callback fired when content changes
- `placeholder: String` - Placeholder text when editor is empty (default: "Start typing...")
- `readonly: bool` - Whether the editor is read-only (default: false)
- `class: String` - Additional CSS classes

### SimpleEditor

A minimal editor variant with basic formatting.

**Props:**
- `initial_value: Option<String>` - Initial content as JSON
- `on_change: EventHandler<String>` - Change callback
- `placeholder: String` - Placeholder text (default: "Write something...")

### ContentViewer

Read-only viewer for rendered content.

**Props:**
- `value: String` - Document content as JSON
- `class: String` - Additional CSS classes

## Data Model

The editor uses an Abstract Syntax Tree (AST) representation:

```rust
pub struct Doc {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub id: String,
    pub kind: BlockKind,
    pub align: BlockAlign,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Inline>,
}

pub enum BlockKind {
    Paragraph,
    Heading { level: u8 },
    BulletList { items: Vec<Block> },
    OrderedList { items: Vec<Block> },
    TaskList { items: Vec<Block> },
    Quote,
    CodeBlock { language: Option<String>, code: String },
    Image { src: String, alt: Option<String>, ... },
    Embed { provider: EmbedProvider, url: String, ... },
    Rule,
}
```

Documents serialize to clean JSON:

```json
{
  "blocks": [
    {
      "id": "uuid-here",
      "kind": {"type": "paragraph"},
      "align": "left",
      "attrs": {},
      "children": [
        {
          "type": "text",
          "text": "Hello world",
          "marks": {"bold": true}
        }
      ]
    }
  ]
}
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+U` | Underline |
| `Ctrl+K` | Insert Link |
| `Enter` | New Paragraph |
| `Shift+Enter` | Line Break |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |

## Security

The editor includes comprehensive HTML sanitization:

- Strips all JavaScript and event handlers
- Sanitizes URLs to prevent `javascript:` protocol attacks
- Allows only safe HTML tags and attributes
- Validates CSS color values
- Auto-adds `rel="noopener noreferrer"` to external links

## Examples

### Blog Post Editor

```rust
#[component]
fn BlogPostEditor() -> Element {
    let mut post_content = use_signal(|| String::new());

    rsx! {
        div { class: "max-w-4xl mx-auto p-6",
            h1 { "Create New Post" }
            
            RichTextEditor {
                initial_value: None,
                on_change: move |content| {
                    post_content.set(content);
                },
                placeholder: "Write your post...".to_string(),
                class: "min-h-[500px]".to_string(),
            }

            button {
                onclick: move |_| {
                    // Save post_content to API
                },
                "Publish Post"
            }
        }
    }
}
```

### Comment Section

```rust
#[component]
fn CommentForm() -> Element {
    let mut comment = use_signal(|| String::new());

    rsx! {
        SimpleEditor {
            initial_value: None,
            on_change: move |content| comment.set(content),
            placeholder: "Write a comment...".to_string(),
        }
        
        button {
            onclick: move |_| {
                // Submit comment
            },
            "Post Comment"
        }
    }
}
```

### Display Rendered Content

```rust
#[component]
fn ArticleView(content: String) -> Element {
    rsx! {
        article { class: "prose max-w-none",
            ContentViewer {
                value: content,
                class: "article-content".to_string(),
            }
        }
    }
}
```

## Module Structure

```
editor/
├── mod.rs           # Main component exports
├── ast.rs           # AST data structures
├── commands.rs      # Editor commands (insert, delete, format)
├── renderer.rs      # AST to HTML rendering
├── sanitizer.rs     # HTML sanitization
├── toolbar.rs       # Toolbar UI component
└── README.md        # This file
```

## Extending

### Custom Commands

Implement the `Command` trait to add custom editing operations:

```rust
use super::commands::{Command, CommandError, Selection};
use super::ast::Doc;

pub struct MyCustomCommand;

impl Command for MyCustomCommand {
    fn execute(&self, doc: &mut Doc, selection: &Selection) 
        -> Result<Selection, CommandError> 
    {
        // Modify the document
        Ok(selection.clone())
    }

    fn description(&self) -> &str {
        "My custom command"
    }
}
```

### Custom Renderers

Override the default HTML rendering:

```rust
use super::renderer::{render_doc, render_block};
use super::ast::Doc;

fn custom_render(doc: &Doc) -> String {
    // Custom rendering logic
    render_doc(doc) // or your own implementation
}
```

## Testing

Run the editor tests:

```bash
cargo test --bin admin-dioxus editor
```

All core functionality is covered by unit tests:
- AST serialization/deserialization
- Command execution
- HTML rendering
- Security sanitization

## Performance

- **Lightweight**: Minimal runtime overhead
- **Efficient Updates**: Only re-renders changed blocks
- **Lazy Rendering**: HTML is memoized and only regenerated on changes
- **Small Bundle**: ~50KB minified (excluding Dioxus core)

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Modern mobile browsers

## Roadmap

- [ ] Undo/Redo history
- [ ] Collaborative editing (OT/CRDT)
- [ ] Tables support
- [ ] File upload integration
- [ ] Markdown import/export
- [ ] Custom plugins API
- [ ] Mobile touch gestures
- [ ] Drag & drop reordering

## License

Same as the parent project.