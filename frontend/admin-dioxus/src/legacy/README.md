# Legacy Code

This directory contains deprecated code that has been replaced with better solutions.

## Editor (Moved Nov 3, 2024)

The custom Dioxus-based rich text editor has been moved here. Building a production-ready editor from scratch in an early-stage framework proved too time-consuming and difficult to maintain.

**What was moved:**
- `editor/` - Complete custom editor implementation (~500KB of code)
  - AST-based document model
  - HTML parser and renderer
  - Command system for formatting
  - Toolbar, bubble menu, slash commands
  - Table support
  - Keyboard shortcuts
  - History management
  - Drag & drop image upload
  - Image editing integration

- `editor_demo.rs` - Editor demo screen
- `rich-text-editor-plan.md` - Original implementation plan

**Replacement:**
Will be replaced with a TypeScript-based editor using battle-tested libraries:
- **Editor.js** (recommended) - Block-based editor with clean JSON output
- **Tiptap** - ProseMirror-based, highly customizable
- **Quill** - Mature, stable WYSIWYG editor

The TypeScript editor will integrate with the Dioxus admin via:
1. Standalone page (separate port) with API integration
2. Or iframe embedding within Dioxus app

This approach saves weeks of development time and provides enterprise-grade features immediately.

**Current Status:**
The blog form now uses a simple textarea temporarily. TypeScript editor integration is pending.

---

## Why This Approach?

Building production-ready editors requires handling:
- Complex cursor/selection management
- Cross-browser compatibility quirks
- Undo/redo with transaction coalescing
- Collaborative editing (eventual)
- Mobile support
- Accessibility (ARIA, screen readers)
- Performance with large documents
- Plugin ecosystem

These are solved problems in mature JS editor libraries. Recreating them in early-stage frameworks is not pragmatic.
