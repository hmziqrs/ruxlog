//! Rich text editor component for Dioxus.
//!
//! This module provides a full-featured WYSIWYG editor with an AST-first architecture.

pub mod ast;
pub mod bubble_menu;
pub mod commands;
pub mod history;
pub mod parser;
pub mod renderer;
pub mod sanitizer;
pub mod shortcuts;
pub mod slash_commands;
pub mod toolbar;

pub use ast::{Block, BlockAlign, BlockKind, Doc, EmbedProvider, Inline, Link, MarkSet, TextSize};
pub use bubble_menu::BubbleMenu;
pub use commands::{Command, CommandError, MarkType, Position, Selection, ToggleMark};
pub use history::{History, Transaction, TransactionType};
pub use parser::parse_html;
pub use renderer::render_doc;
pub use sanitizer::sanitize_html;
pub use shortcuts::{format_shortcut, Shortcut, ShortcutAction, ShortcutRegistry};
pub use slash_commands::{SlashCommand, SlashCommands};
pub use toolbar::LinkDialog;

use commands::{InsertBlock, InsertLink, SetBlockType};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::JsCast;
use crate::components::image_editor::ImageEditorModal;
use crate::store::{use_media, use_image_editor, MediaReference, MediaUploadPayload};

static EDITOR_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Props for the RichTextEditor component.
#[derive(Props, Clone, PartialEq)]
pub struct RichTextEditorProps {
    /// Initial document content (JSON).
    #[props(default)]
    pub initial_value: Option<String>,

    /// Callback when content changes.
    pub on_change: EventHandler<String>,

    /// Placeholder text when editor is empty.
    #[props(default = "Start typing...".to_string())]
    pub placeholder: String,

    /// Whether the editor is read-only.
    #[props(default = false)]
    pub readonly: bool,

    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Main rich text editor component.
#[component]
pub fn RichTextEditor(props: RichTextEditorProps) -> Element {
    // Unique ID for this editor instance
    let editor_id = use_signal(|| {
        format!(
            "rich-text-editor-{}",
            EDITOR_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    });

    // Initial HTML content from props (computed once)
    let initial_html = {
        if let Some(s) = &props.initial_value {
            // Try AST JSON first; fall back to raw HTML
            match serde_json::from_str::<Doc>(s) {
                Ok(doc) => render_doc(&doc),
                Err(_) => sanitize_html(s),
            }
        } else {
            String::new()
        }
    };

    let mut selection = use_signal(|| Selection::collapsed(Position::start()));
    let mut is_focused = use_signal(|| false);
    let mut show_bubble_menu = use_signal(|| false);
    let mut show_slash_menu = use_signal(|| false);
    let mut slash_query = use_signal(|| String::new());
    let mut show_link_dialog = use_signal(|| false);
    let mut is_dragging_over = use_signal(|| false);
    let mut uploading_images = use_signal(|| Vec::<String>::new());

    // Custom undo/redo history manager
    let mut edit_history = use_signal(|| History::new(initial_html.clone()));

    // Track if we're in the middle of an undo/redo operation
    let mut is_undoing = use_signal(|| false);

    // Screen reader announcements
    let mut sr_announcement = use_signal(|| String::new());

    // Image editor integration
    let image_editor = use_image_editor();
    let mut editing_image_src = use_signal(|| Option::<String>::None);

    // Keyboard shortcuts registry
    let shortcuts = use_signal(|| ShortcutRegistry::with_defaults());

    // Media state for uploads
    let media_state = use_media();

    // Initialize JavaScript bridge on mount
    use_effect(move || {
        let id = editor_id();
        let js_init = format!(
            r#"
            if (typeof window.editorDragDropBridge === 'undefined') {{
                {}
            }}
            window.editorDragDropBridge.captureDragEvent('{}');
            "#,
            include_str!("drag_drop_bridge.js"),
            id
        );
        let _ = js_sys::eval(&js_init);
    });

    // Execute a command using browser's native execCommand
    let execute_command = move |cmd: Box<dyn Command>| {
        let Some(document) = current_document() else {
            return;
        };

        if let Some(toggle_mark) = cmd.as_any().downcast_ref::<ToggleMark>() {
            if handle_toggle_mark(&document, toggle_mark.mark_type) {
                return;
            }
        }

        if let Some(set_block) = cmd.as_any().downcast_ref::<SetBlockType>() {
            handle_set_block_type(&document, &set_block.block_type);
            return;
        }

        if let Some(insert_block) = cmd.as_any().downcast_ref::<InsertBlock>() {
            handle_insert_block(&document, &insert_block.block_type);
            return;
        }

        if let Some(insert_link) = cmd.as_any().downcast_ref::<InsertLink>() {
            handle_insert_link(&document, insert_link);
            return;
        }

        if let Some(insert_table) = cmd.as_any().downcast_ref::<commands::InsertTable>() {
            gloo_console::log!("[Editor] InsertTable command received: rows=", insert_table.rows, "cols=", insert_table.cols);
            let headers = vec![vec![Inline::text("")]; insert_table.cols];
            let rows = vec![vec![vec![Inline::text("")]; insert_table.cols]; insert_table.rows];
            let column_align = vec![ast::TableAlign::default(); insert_table.cols];
            gloo_console::log!("[Editor] Calling handle_insert_block with table");
            handle_insert_block(
                &document,
                &BlockKind::Table {
                    headers,
                    rows,
                    column_align,
                },
            );
            gloo_console::log!("[Editor] Table inserted successfully");
            return;
        }
    };

    // Set initial content on mount
    use_effect(move || {
        let id = editor_id();
        if !initial_html.is_empty() {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(element) = document.get_element_by_id(&id) {
                        let _ = element.set_inner_html(&initial_html);
                    }
                }
            }
        }
    });

    // Set up double-click handler for images to open editor
    use_effect(move || {
        let id = editor_id();
        let js_setup_image_dblclick = format!(
            r#"
            (function() {{
                const editor = document.getElementById('{}');
                if (!editor) return;

                // Remove existing listener if any
                if (editor._imageEditHandler) {{
                    editor.removeEventListener('dblclick', editor._imageEditHandler);
                }}

                // Add new listener
                editor._imageEditHandler = async function(e) {{
                    if (e.target.tagName === 'IMG') {{
                        e.preventDefault();
                        e.stopPropagation();

                        const imageSrc = e.target.src;
                        console.log('[RichTextEditor] Image double-clicked:', imageSrc);

                        // Store the image element for later replacement
                        window._editingImageElement = e.target;

                        // Trigger opening the image editor
                        // This will be detected by checking window._editingImageElement in Rust
                        window._pendingImageEdit = imageSrc;
                    }}
                }};

                editor.addEventListener('dblclick', editor._imageEditHandler);
            }})()
            "#,
            id
        );
        let _ = js_sys::eval(&js_setup_image_dblclick);
    });

    // Poll for pending image edits from JavaScript
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;

                // Check if there's a pending image edit
                let js_check_pending = "window._pendingImageEdit || null";
                if let Ok(pending_val) = js_sys::eval(js_check_pending) {
                    if let Some(src) = pending_val.as_string() {
                        gloo_console::log!("[RichTextEditor] Opening image editor for:", &src);

                        // Clear the pending flag
                        let _ = js_sys::eval("window._pendingImageEdit = null");

                        editing_image_src.set(Some(src.clone()));

                        // Fetch the image and create a blob URL
                        if let Some(window) = web_sys::window() {
                            match wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&src)).await {
                                Ok(response) => {
                                    if let Ok(response) = response.dyn_into::<web_sys::Response>() {
                                        if let Ok(blob_promise) = response.blob() {
                                            match wasm_bindgen_futures::JsFuture::from(blob_promise).await {
                                                Ok(blob) => {
                                                    if let Ok(blob) = blob.dyn_into::<web_sys::Blob>() {
                                                        if let Ok(blob_url) = web_sys::Url::create_object_url_with_blob(&blob) {
                                                            gloo_console::log!("[RichTextEditor] Opening image editor with blob URL:", &blob_url);

                                                            if let Err(e) = image_editor.open_editor(None, blob_url).await {
                                                                gloo_console::error!("[RichTextEditor] Failed to open image editor:", &e);
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    gloo_console::error!("[RichTextEditor] Failed to get blob:", format!("{:?}", e));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    gloo_console::error!("[RichTextEditor] Failed to fetch image:", format!("{:?}", e));
                                }
                            }
                        }
                    }
                }
            }
        });
    });

    // Track selection changes for bubble menu
    let update_selection_mouse = move |_evt: Event<MouseData>| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(sel)) = window.get_selection() {
                let has_selection = sel
                    .to_string()
                    .as_string()
                    .map_or(false, |s| !s.trim().is_empty());

                // Update selection state (using collapsed or simple representation)
                if has_selection {
                    selection.set(Selection::new(Position::start(), Position::new(0, 1)));
                } else {
                    selection.set(Selection::collapsed(Position::start()));
                }
                show_bubble_menu.set(has_selection && is_focused());
            }
        }
    };

    let update_selection_keyboard = move |evt: Event<KeyboardData>| {
        // Check for keyboard shortcuts first
        if let Some(action) = shortcuts.read().find_action(&evt) {
            evt.prevent_default();

            match action {
                ShortcutAction::ToggleBold => {
                    execute_command(Box::new(ToggleMark {
                        mark_type: MarkType::Bold,
                    }));
                }
                ShortcutAction::ToggleItalic => {
                    execute_command(Box::new(ToggleMark {
                        mark_type: MarkType::Italic,
                    }));
                }
                ShortcutAction::ToggleUnderline => {
                    execute_command(Box::new(ToggleMark {
                        mark_type: MarkType::Underline,
                    }));
                }
                ShortcutAction::ToggleStrike => {
                    execute_command(Box::new(ToggleMark {
                        mark_type: MarkType::Strike,
                    }));
                }
                ShortcutAction::ToggleCode => {
                    execute_command(Box::new(ToggleMark {
                        mark_type: MarkType::Code,
                    }));
                }
                ShortcutAction::InsertLink => {
                    show_link_dialog.set(true);
                }
                ShortcutAction::SetHeading(level) => {
                    if let Some(document) = current_document() {
                        handle_set_block_type(&document, &BlockKind::Heading { level });
                    }
                }
                ShortcutAction::SetParagraph => {
                    if let Some(document) = current_document() {
                        handle_set_block_type(&document, &BlockKind::Paragraph);
                    }
                }
                ShortcutAction::SetQuote => {
                    if let Some(document) = current_document() {
                        handle_set_block_type(&document, &BlockKind::Quote);
                    }
                }
                ShortcutAction::SetCodeBlock => {
                    if let Some(document) = current_document() {
                        handle_set_block_type(
                            &document,
                            &BlockKind::CodeBlock {
                                language: None,
                                code: String::new(),
                            },
                        );
                    }
                }
                ShortcutAction::InsertBulletList => {
                    let _ =
                        js_sys::eval("document.execCommand('insertUnorderedList', false, null)");
                }
                ShortcutAction::InsertOrderedList => {
                    let _ = js_sys::eval("document.execCommand('insertOrderedList', false, null)");
                }
                ShortcutAction::InsertTaskList => {
                    if let Some(document) = current_document() {
                        handle_insert_block(&document, &BlockKind::TaskItem { checked: false });
                    }
                }
                ShortcutAction::Undo => {
                    // Use custom history instead of browser's native undo
                    if let Some(restored_html) = edit_history.write().undo() {
                        is_undoing.set(true);

                        // Update the editor content
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(element) = document.get_element_by_id(&editor_id()) {
                                    let _ = element.set_inner_html(&restored_html);
                                }
                            }
                        }

                        // Notify parent of change
                        props.on_change.call(sanitize_html(&restored_html));

                        is_undoing.set(false);
                        sr_announcement.set("Undo applied".to_string());
                        gloo_console::log!("[RichTextEditor] Undo applied");
                    } else {
                        sr_announcement.set("Nothing to undo".to_string());
                    }
                }
                ShortcutAction::Redo => {
                    // Use custom history instead of browser's native redo
                    if let Some(restored_html) = edit_history.write().redo() {
                        is_undoing.set(true);

                        // Update the editor content
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(element) = document.get_element_by_id(&editor_id()) {
                                    let _ = element.set_inner_html(&restored_html);
                                }
                            }
                        }

                        // Notify parent of change
                        props.on_change.call(sanitize_html(&restored_html));

                        is_undoing.set(false);
                        sr_announcement.set("Redo applied".to_string());
                        gloo_console::log!("[RichTextEditor] Redo applied");
                    } else {
                        sr_announcement.set("Nothing to redo".to_string());
                    }
                }
                ShortcutAction::Save => {
                    // Trigger save event - parent component can handle this
                    gloo_console::log!("Save shortcut triggered");
                }
                ShortcutAction::Find => {
                    // Could trigger find dialog in the future
                    gloo_console::log!("Find shortcut triggered");
                }
                ShortcutAction::MoveBlockUp => {
                    // Move the current block up in the DOM
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            // Get the selection to find current block
                            if let Some(sel) = document.get_selection().ok().flatten() {
                                if let Some(anchor_node) = sel.anchor_node() {
                                    // Find the parent block element
                                    if let Some(block) = find_parent_block(&anchor_node, &document) {
                                        // Get the previous sibling block
                                        if let Some(prev_block) = block.previous_element_sibling() {
                                            // Swap positions
                                            if let Some(parent) = block.parent_element() {
                                                let _ = parent.insert_before(&block, Some(&prev_block));

                                                // Add to history
                                                if let Some(element) = document.get_element_by_id(&editor_id()) {
                                                    let new_html = element.inner_html();
                                                    let clean = sanitize_html(&new_html);
                                                    edit_history.write().push(clean.clone(), TransactionType::BlockChange);
                                                    props.on_change.call(clean);
                                                }

                                                sr_announcement.set("Block moved up".to_string());
                                                gloo_console::log!("[RichTextEditor] Moved block up");
                                            }
                                        } else {
                                            sr_announcement.set("Cannot move block up, already at top".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ShortcutAction::MoveBlockDown => {
                    // Move the current block down in the DOM
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            // Get the selection to find current block
                            if let Some(sel) = document.get_selection().ok().flatten() {
                                if let Some(anchor_node) = sel.anchor_node() {
                                    // Find the parent block element
                                    if let Some(block) = find_parent_block(&anchor_node, &document) {
                                        // Get the next sibling block
                                        if let Some(next_block) = block.next_element_sibling() {
                                            // Get the block after next (for insertion point)
                                            let after_next = next_block.next_element_sibling();

                                            // Swap positions
                                            if let Some(parent) = block.parent_element() {
                                                // Cast Element to Node for insert_before
                                                let after_next_node = after_next.as_ref().map(|e| e as &web_sys::Node);
                                                let _ = parent.insert_before(&block, after_next_node);

                                                // Add to history
                                                if let Some(element) = document.get_element_by_id(&editor_id()) {
                                                    let new_html = element.inner_html();
                                                    let clean = sanitize_html(&new_html);
                                                    edit_history.write().push(clean.clone(), TransactionType::BlockChange);
                                                    props.on_change.call(clean);
                                                }

                                                sr_announcement.set("Block moved down".to_string());
                                                gloo_console::log!("[RichTextEditor] Moved block down");
                                            }
                                        } else {
                                            sr_announcement.set("Cannot move block down, already at bottom".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            return;
        }

        if let Some(window) = web_sys::window() {
            if let Ok(Some(sel)) = window.get_selection() {
                let has_selection = sel
                    .to_string()
                    .as_string()
                    .map_or(false, |s| !s.trim().is_empty());

                // Update selection state (using collapsed or simple representation)
                if has_selection {
                    selection.set(Selection::new(Position::start(), Position::new(0, 1)));
                } else {
                    selection.set(Selection::collapsed(Position::start()));
                }
                show_bubble_menu.set(has_selection && is_focused());

                // Check for slash command trigger
                let key = evt.key();

                // Detect slash key
                if key == Key::Character("/".to_string()) && !has_selection {
                    show_slash_menu.set(true);
                    slash_query.set(String::new());
                } else if show_slash_menu() {
                    // Update query as user types after '/'
                    if key == Key::Escape || key == Key::Enter {
                        show_slash_menu.set(false);
                        slash_query.set(String::new());
                    } else if key == Key::Backspace {
                        let current = slash_query();
                        if current.is_empty() {
                            show_slash_menu.set(false);
                        } else {
                            let mut chars: Vec<char> = current.chars().collect();
                            chars.pop();
                            slash_query.set(chars.into_iter().collect());
                        }
                    } else if let Key::Character(ch) = key {
                        // Add typed character to query
                        slash_query.set(format!("{}{}", slash_query(), ch));
                    }
                }
            }
        }
    };

    // Handle slash command selection
    let handle_slash_select = move |cmd: SlashCommand| {
        show_slash_menu.set(false);
        slash_query.set(String::new());

        // Remove the '/' and query text
        if let Some(document) = current_document() {
            if let Ok(Some(sel)) = document.get_selection() {
                if sel.range_count() > 0 {
                    if let Ok(_range) = sel.get_range_at(0) {
                        // Delete backwards to remove '/' and query
                        let delete_count = 1 + slash_query().len();
                        for _ in 0..delete_count {
                            let _ = js_sys::eval("document.execCommand('delete', false, null)");
                        }
                    }
                }
            }
        }

        // Execute appropriate command based on selection
        match cmd {
            SlashCommand::Paragraph => {
                if let Some(document) = current_document() {
                    handle_set_block_type(&document, &BlockKind::Paragraph);
                }
            }
            SlashCommand::Heading1 => {
                if let Some(document) = current_document() {
                    handle_set_block_type(&document, &BlockKind::Heading { level: 1 });
                }
            }
            SlashCommand::Heading2 => {
                if let Some(document) = current_document() {
                    handle_set_block_type(&document, &BlockKind::Heading { level: 2 });
                }
            }
            SlashCommand::Heading3 => {
                if let Some(document) = current_document() {
                    handle_set_block_type(&document, &BlockKind::Heading { level: 3 });
                }
            }
            SlashCommand::BulletList => {
                let _ = js_sys::eval("document.execCommand('insertUnorderedList', false, null)");
            }
            SlashCommand::OrderedList => {
                let _ = js_sys::eval("document.execCommand('insertOrderedList', false, null)");
            }
            SlashCommand::TaskList => {
                if let Some(document) = current_document() {
                    handle_insert_block(&document, &BlockKind::TaskItem { checked: false });
                }
            }
            SlashCommand::Quote => {
                if let Some(document) = current_document() {
                    handle_set_block_type(&document, &BlockKind::Quote);
                }
            }
            SlashCommand::CodeBlock => {
                if let Some(document) = current_document() {
                    handle_set_block_type(
                        &document,
                        &BlockKind::CodeBlock {
                            language: None,
                            code: String::new(),
                        },
                    );
                }
            }
            SlashCommand::Divider => {
                if let Some(document) = current_document() {
                    handle_insert_block(&document, &BlockKind::Rule);
                }
            }
            SlashCommand::Image => {
                if let Some(document) = current_document() {
                    handle_insert_block(
                        &document,
                        &BlockKind::Image {
                            src: String::from(""),
                            alt: None,
                            title: None,
                            width: None,
                            height: None,
                            caption: None,
                        },
                    );
                }
            }
            SlashCommand::Table => {
                if let Some(document) = current_document() {
                    // Insert a default 3x3 table
                    let headers = vec![vec![Inline::text("")]; 3];
                    let rows = vec![vec![vec![Inline::text("")]; 3]; 3];
                    let column_align = vec![ast::TableAlign::default(); 3];
                    handle_insert_block(
                        &document,
                        &BlockKind::Table {
                            headers,
                            rows,
                            column_align,
                        },
                    );
                }
            }
        }
    };

    // Drag-and-drop event handlers
    let handle_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
        is_dragging_over.set(true);
    };

    let handle_drag_enter = move |evt: Event<DragData>| {
        evt.prevent_default();
        is_dragging_over.set(true);
    };

    let handle_drag_leave = move |evt: Event<DragData>| {
        evt.prevent_default();
        is_dragging_over.set(false);
    };

    let handle_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        is_dragging_over.set(false);

        if props.readonly {
            return;
        }

        // Use JavaScript bridge to access files
        // The bridge captures the native drop event and exposes the files
        let js_get_files = r#"
            (function() {
                const files = window.editorDragDropBridge.getFilesFromLastDrop();
                return window.editorDragDropBridge.processFiles(files);
            })()
        "#;

        let files_result = js_sys::eval(js_get_files);
        if files_result.is_err() {
            gloo_console::error!("[RichTextEditor] Failed to access dropped files");
            return;
        }

        let files_val = files_result.unwrap();
        let files_array = js_sys::Array::from(&files_val);

        if files_array.length() == 0 {
            gloo_console::warn!("[RichTextEditor] No files in drop event");
            return;
        }

        gloo_console::log!("[RichTextEditor] Processing", files_array.length().to_string(), "dropped files");

        // Process each file
        for i in 0..files_array.length() {
            let file_obj = files_array.get(i);
            let file_obj = file_obj.dyn_into::<js_sys::Object>().unwrap();

            // Extract properties from the file object
            let blob_url = js_sys::Reflect::get(&file_obj, &"blobUrl".into())
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            let file_type = js_sys::Reflect::get(&file_obj, &"type".into())
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            // Only process image files
            if !file_type.starts_with("image/") {
                gloo_console::warn!("[RichTextEditor] Skipping non-image file:", &file_type);
                continue;
            }

            // Get the native File object for upload
            let native_file = js_sys::Reflect::get(&file_obj, &"file".into())
                .ok()
                .and_then(|v| v.dyn_into::<web_sys::File>().ok());

            if native_file.is_none() {
                gloo_console::error!("[RichTextEditor] Could not extract File object");
                continue;
            }

            let file = native_file.unwrap();

            // Add to uploading list
            let mut current_uploading = uploading_images();
            current_uploading.push(blob_url.clone());
            uploading_images.set(current_uploading);

            // Insert placeholder image immediately
            if let Some(document) = current_document() {
                let placeholder_html = format!(
                    r#"<div class="upload-placeholder" data-blob-url="{}">
                        <img src="{}" alt="Uploading..." class="editor-image max-w-full rounded-md opacity-50" />
                        <div class="upload-progress">Uploading...</div>
                    </div>"#,
                    blob_url, blob_url
                );
                insert_html(&document, &placeholder_html);
            }

            // Upload the file in the background
            let media_state_ref = media_state;
            let blob_url_clone = blob_url.clone();
            let mut uploading_images_clone = uploading_images.clone();

            spawn(async move {
                let payload = MediaUploadPayload {
                    file,
                    reference_type: Some(MediaReference::Post),
                    width: None,
                    height: None,
                };

                match media_state_ref.upload(payload).await {
                    Ok(upload_blob_url) => {
                        gloo_console::log!("[RichTextEditor] Upload successful:", &upload_blob_url);

                        // Poll for the actual media URL (the upload is async)
                        // We need to wait a bit for the server to process
                        gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;

                        let media_url = {
                            let blob_to_media = media_state_ref.blob_to_media.read();
                            if let Some(Some(media)) = blob_to_media.get(&upload_blob_url) {
                                Some(media.file_url.clone())
                            } else {
                                None
                            }
                        };

                        if let Some(actual_url) = media_url {
                            // Replace placeholder with actual image
                            if let Some(window) = web_sys::window() {
                                if let Some(document) = window.document() {
                                    let selector = format!("[data-blob-url='{}']", blob_url_clone);
                                    if let Ok(Some(placeholder)) = document.query_selector(&selector) {
                                        let img_html = format!(
                                            r#"<img src="{}" alt="" class="editor-image max-w-full rounded-md" loading="lazy" />"#,
                                            actual_url
                                        );
                                        placeholder.set_outer_html(&img_html);
                                    }
                                }
                            }
                        }

                        // Remove from uploading list
                        let mut current = uploading_images_clone();
                        current.retain(|url| url != &blob_url_clone);
                        uploading_images_clone.set(current);
                    }
                    Err(err) => {
                        gloo_console::error!("[RichTextEditor] Upload failed:", &err);

                        // Remove placeholder on error
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                let selector = format!("[data-blob-url='{}']", blob_url_clone);
                                if let Ok(Some(placeholder)) = document.query_selector(&selector) {
                                    let _ = placeholder.remove();
                                }
                            }
                        }

                        // Remove from uploading list
                        let mut current = uploading_images_clone();
                        current.retain(|url| url != &blob_url_clone);
                        uploading_images_clone.set(current);
                    }
                }
            });
        }
    };

    // Image editor: Handle save callback
    let handle_image_editor_save = move |edited_file: web_sys::File| {
        gloo_console::log!("[RichTextEditor] Image editor saved, uploading edited image...");

        let media_state_ref = media_state;
        let mut editing_image_src_clone = editing_image_src.clone();

        spawn(async move {
            // Upload the edited file
            let payload = MediaUploadPayload {
                file: edited_file,
                reference_type: Some(MediaReference::Post),
                width: None,
                height: None,
            };

            match media_state_ref.upload(payload).await {
                Ok(_upload_blob_url) => {
                    gloo_console::log!("[RichTextEditor] Edited image uploaded, refreshing media list...");

                    // Wait a bit for server processing
                    gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;

                    // Refresh media list to get the new URL
                    let _ = media_state_ref.list().await;

                    // Get the latest media item (should be our upload)
                    let media_url = {
                        let list_state = media_state_ref.list.read();
                        list_state.data.as_ref()
                            .and_then(|paginated| paginated.data.first())
                            .map(|m| m.file_url.clone())
                    };

                    if let Some(new_url) = media_url {
                        gloo_console::log!("[RichTextEditor] Replacing image with:", &new_url);

                        // Replace the image in the editor using the stored element reference
                        let js_replace = format!(
                            r#"
                            (function() {{
                                const imgElement = window._editingImageElement;
                                if (imgElement) {{
                                    imgElement.src = '{}';
                                    window._editingImageElement = null;
                                }}
                            }})()
                            "#,
                            new_url
                        );

                        let _ = js_sys::eval(&js_replace);

                        // Clear the editing state
                        editing_image_src_clone.set(None);
                    } else {
                        gloo_console::error!("[RichTextEditor] Failed to get media URL after upload");
                    }
                }
                Err(e) => {
                    gloo_console::error!("[RichTextEditor] Failed to upload edited image:", &e.to_string());
                }
            }
        });
    };

    // Compute editor content class
    let editor_content_class = if *is_dragging_over.read() {
        "editor-content min-h-[300px] focus:outline-none drag-over"
    } else {
        "editor-content min-h-[300px] focus:outline-none"
    };

    rsx! {
        div {
            class: "rich-text-editor border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800 shadow-sm {props.class}",

            // Toolbar
            if !props.readonly {
                toolbar::Toolbar {
                    on_command: move |cmd| execute_command(cmd),
                    selection: selection.read().clone(),
                }
            }

            // Editor content area - simple contenteditable without dangerous_inner_html
            div {
                id: "{editor_id}",
                class: "{editor_content_class}",
                contenteditable: if props.readonly { "false" } else { "true" },
                tabindex: "0",
                "data-placeholder": "{props.placeholder}",
                role: "textbox",
                "aria-label": "Rich text editor",
                "aria-multiline": "true",
                "aria-readonly": if props.readonly { "true" } else { "false" },

                style: "
                    padding: 1rem;
                    white-space: pre-wrap;
                    word-wrap: break-word;
                    line-height: 1.6;
                    color: inherit;
                ",

                onfocus: move |_| {
                    is_focused.set(true);
                },

                onblur: move |_| {
                    is_focused.set(false);
                    show_bubble_menu.set(false);
                },

                onmouseup: update_selection_mouse,
                onkeyup: update_selection_keyboard,

                ondragover: handle_drag_over,
                ondragenter: handle_drag_enter,
                ondragleave: handle_drag_leave,
                ondrop: handle_drop,

                onpaste: move |evt: Event<ClipboardData>| {
                    if props.readonly {
                        return;
                    }

                    // Prevent default paste to handle it ourselves
                    evt.prevent_default();

                    // Use JavaScript bridge to get clipboard data
                    // The bridge captures the native paste event via event listener
                    let js_get_clipboard = "window.editorDragDropBridge.getClipboardData()";

                    match js_sys::eval(js_get_clipboard) {
                        Ok(clip_data) => {
                            // Parse the clipboard data
                            let clip_type = js_sys::Reflect::get(&clip_data, &"type".into())
                                .ok()
                                .and_then(|v| v.as_string())
                                .unwrap_or_default();

                            let clip_content = js_sys::Reflect::get(&clip_data, &"content".into())
                                .ok()
                                .and_then(|v| v.as_string())
                                .unwrap_or_default();

                            if clip_content.is_empty() || clip_type == "empty" {
                                return;
                            }

                            // Process based on type
                            let sanitized_content = if clip_type == "html" {
                                gloo_console::log!("[RichTextEditor] Pasting HTML content (preserving formatting)");
                                // Sanitize the HTML to remove unwanted tags/attributes
                                // This preserves formatting from Word/Google Docs while removing unsafe content
                                sanitize_html(&clip_content)
                            } else {
                                gloo_console::log!("[RichTextEditor] Pasting plain text");
                                // Escape HTML entities in plain text and preserve line breaks
                                clip_content.replace('&', "&amp;")
                                    .replace('<', "&lt;")
                                    .replace('>', "&gt;")
                                    .replace('"', "&quot;")
                                    .replace('\n', "<br>")
                            };

                            // Insert the sanitized content at the cursor
                            if let Some(document) = current_document() {
                                insert_html(&document, &sanitized_content);

                                // Manually trigger history entry for paste operation
                                // Get the updated content after paste
                                if let Some(window) = web_sys::window() {
                                    if let Some(doc) = window.document() {
                                        if let Some(element) = doc.get_element_by_id(&editor_id()) {
                                            let new_html = element.inner_html();
                                            let clean = sanitize_html(&new_html);
                                            edit_history.write().push(clean, TransactionType::Paste);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            gloo_console::error!("[RichTextEditor] Failed to access clipboard:", format!("{:?}", e));
                        }
                    }
                },

                oninput: move |evt| {
                    if props.readonly {
                        return;
                    }

                    if let Some(document) = current_document() {
                        sync_task_checkbox_state(&document);
                    }

                    // Get the HTML content from contenteditable
                    let html_value = evt.value();

                    // Sanitize
                    let clean = sanitize_html(&html_value);

                    // Add to history (unless we're in the middle of undo/redo)
                    if !*is_undoing.read() {
                        // Determine transaction type based on content change
                        // Simple heuristic: if content length difference is small, it's typing
                        let current_len = edit_history.read().current_state().len();
                        let new_len = clean.len();
                        let len_diff = (new_len as i32 - current_len as i32).abs();

                        let transaction_type = if len_diff <= 5 {
                            // Small change - likely typing or delete
                            if new_len > current_len {
                                TransactionType::Typing
                            } else {
                                TransactionType::Delete
                            }
                        } else {
                            // Large change - likely paste or formatting
                            TransactionType::Other
                        };

                        edit_history.write().push(clean.clone(), transaction_type);
                    }

                    // Notify parent
                    props.on_change.call(clean);
                },
            }

            // Bubble menu for quick formatting
            if !props.readonly {
                BubbleMenu {
                    show: *show_bubble_menu.read(),
                    selection: selection.read().clone(),
                    on_command: move |cmd| execute_command(cmd),
                    editor_id: editor_id(),
                }
            }

            // Slash commands menu
            if !props.readonly {
                SlashCommands {
                    show: *show_slash_menu.read(),
                    on_select: handle_slash_select,
                    on_close: move |_| {
                        show_slash_menu.set(false);
                        slash_query.set(String::new());
                    },
                    editor_id: editor_id(),
                    query: slash_query(),
                }
            }

            // Link dialog (triggered by keyboard shortcut)
            if !props.readonly && *show_link_dialog.read() {
                LinkDialog {
                    on_close: move |_| show_link_dialog.set(false),
                    on_insert: move |(href, title)| {
                        execute_command(Box::new(InsertLink {
                            href,
                            title,
                            target_blank: true,
                        }));
                        show_link_dialog.set(false);
                    },
                }
            }

            // Image editor modal
            if !props.readonly {
                ImageEditorModal {
                    on_save: handle_image_editor_save,
                }
            }

            // Inline styles for placeholder, drag-drop, and upload progress
            style {
                r"
                .editor-content[data-placeholder]:empty:before {{
                    content: attr(data-placeholder);
                    color: #9ca3af;
                    pointer-events: none;
                }}
                .dark .editor-content[data-placeholder]:empty:before {{
                    color: #6b7280;
                }}
                .editor-content:focus {{
                    outline: 2px solid #3b82f6;
                    outline-offset: -2px;
                }}
                .dark .editor-content:focus {{
                    outline-color: #60a5fa;
                }}
                .editor-content.drag-over {{
                    background-color: rgba(59, 130, 246, 0.05);
                    border: 2px dashed #3b82f6;
                }}
                .dark .editor-content.drag-over {{
                    background-color: rgba(96, 165, 250, 0.05);
                    border-color: #60a5fa;
                }}
                .upload-placeholder {{
                    position: relative;
                    display: inline-block;
                    margin: 1rem 0;
                }}
                .upload-progress {{
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    background: rgba(0, 0, 0, 0.7);
                    color: white;
                    padding: 0.5rem 1rem;
                    border-radius: 0.375rem;
                    font-size: 0.875rem;
                    pointer-events: none;
                }}
                .dark .upload-progress {{
                    background: rgba(255, 255, 255, 0.2);
                    backdrop-filter: blur(4px);
                }}
                .editor-content ul {{
                    list-style-type: disc;
                    padding-left: 2rem;
                    margin: 1rem 0;
                }}
                .editor-content ol {{
                    list-style-type: decimal;
                    padding-left: 2rem;
                    margin: 1rem 0;
                }}
                .editor-content li {{
                    margin: 0.5rem 0;
                }}
                .editor-content .task-list {{
                    list-style: none;
                    padding-left: 0;
                    margin: 0;
                }}
                .editor-content .task-list-item {{
                    display: flex;
                    align-items: flex-start;
                    gap: 0.5rem;
                    margin: 0.25rem 0;
                }}
                .editor-content .task-checkbox {{
                    margin-top: 0.35rem;
                    width: 1rem;
                    height: 1rem;
                }}
                .dark .editor-content .task-checkbox {{
                    accent-color: #60a5fa;
                }}
                .editor-content h1,
                .editor-content h2,
                .editor-content h3,
                .editor-content h4,
                .editor-content h5,
                .editor-content h6 {{
                    font-weight: bold;
                    margin: 1rem 0;
                }}
                .editor-content h1 {{ font-size: 2rem; }}
                .editor-content h2 {{ font-size: 1.5rem; }}
                .editor-content h3 {{ font-size: 1.25rem; }}
                .editor-content h4 {{ font-size: 1.1rem; }}
                .editor-content h5 {{ font-size: 1rem; }}
                .editor-content h6 {{ font-size: 0.9rem; }}
                .editor-content p {{
                    margin: 0.5rem 0;
                }}
                .editor-content blockquote {{
                    border-left: 4px solid #d1d5db;
                    padding-left: 1rem;
                    margin: 1rem 0;
                    font-style: italic;
                    color: #6b7280;
                }}
                .dark .editor-content blockquote {{
                    border-left-color: #4b5563;
                    color: #9ca3af;
                }}
                .editor-content pre {{
                    background-color: #f3f4f6;
                    padding: 1rem;
                    border-radius: 0.375rem;
                    overflow-x: auto;
                    margin: 1rem 0;
                }}
                .dark .editor-content pre {{
                    background-color: #1f2937;
                }}
                .editor-content code {{
                    font-family: monospace;
                }}
                /* Image editing styles */
                .editor-content img {{
                    max-width: 100%;
                    height: auto;
                    border-radius: 0.375rem;
                    cursor: pointer;
                    transition: all 0.2s ease-in-out;
                }}
                .editor-content img:hover {{
                    opacity: 0.9;
                    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
                    outline: 2px solid #3b82f6;
                    outline-offset: 2px;
                }}
                .dark .editor-content img:hover {{
                    outline-color: #60a5fa;
                }}
                /* Block reordering drag handles */
                .editor-content > * {{
                    position: relative;
                }}
                .editor-content > *:hover .drag-handle {{
                    opacity: 1;
                }}
                .drag-handle {{
                    position: absolute;
                    left: -2rem;
                    top: 50%;
                    transform: translateY(-50%);
                    cursor: grab;
                    opacity: 0;
                    transition: opacity 0.2s;
                    padding: 0.25rem;
                    color: #9ca3af;
                }}
                .dark .drag-handle {{
                    color: #6b7280;
                }}
                .drag-handle:hover {{
                    color: #3b82f6;
                }}
                .dark .drag-handle:hover {{
                    color: #60a5fa;
                }}
                .drag-handle:active {{
                    cursor: grabbing;
                }}
                .block-dragging {{
                    opacity: 0.5;
                    background: rgba(59, 130, 246, 0.1);
                }}
                .drop-indicator {{
                    height: 2px;
                    background: #3b82f6;
                    margin: 0.5rem 0;
                }}
                /* Screen reader only - visually hidden but accessible */
                .sr-only {{
                    position: absolute;
                    width: 1px;
                    height: 1px;
                    padding: 0;
                    margin: -1px;
                    overflow: hidden;
                    clip: rect(0, 0, 0, 0);
                    white-space: nowrap;
                    border-width: 0;
                }}
                "
            }

            // Screen reader announcements (visually hidden)
            div {
                class: "sr-only",
                role: "status",
                "aria-live": "polite",
                "aria-atomic": "true",
                "{sr_announcement.read()}"
            }

            // Character count (optional)
            if *is_focused.read() {
                div {
                    class: "px-4 py-2 text-xs text-gray-500 dark:text-gray-400 border-t border-gray-200 dark:border-gray-700",
                    "Type to edit..."
                }
            }
        }
    }
}

/// Simple editor variant with minimal toolbar.
#[component]
pub fn SimpleEditor(
    #[props(default)] initial_value: Option<String>,
    on_change: EventHandler<String>,
    #[props(default = "Write something...".to_string())] placeholder: String,
) -> Element {
    rsx! {
        RichTextEditor {
            initial_value,
            on_change,
            placeholder,
            class: "simple-editor",
        }
    }
}

fn current_document() -> Option<web_sys::Document> {
    web_sys::window().and_then(|window| window.document())
}

fn exec_command(document: &web_sys::Document, command: &str) {
    if let Some(html_document) = document.dyn_ref::<web_sys::HtmlDocument>() {
        let _ = html_document.exec_command(command);
    }
}

fn exec_command_with_value(document: &web_sys::Document, command: &str, value: &str) {
    if let Some(html_document) = document.dyn_ref::<web_sys::HtmlDocument>() {
        let _ = html_document.exec_command_with_show_ui_and_value(command, false, value);
    }
}

fn handle_toggle_mark(document: &web_sys::Document, mark: MarkType) -> bool {
    match mark {
        MarkType::Bold => {
            exec_command(document, "bold");
            true
        }
        MarkType::Italic => {
            exec_command(document, "italic");
            true
        }
        MarkType::Underline => {
            exec_command(document, "underline");
            true
        }
        MarkType::Strike => {
            exec_command(document, "strikeThrough");
            true
        }
        MarkType::Code => {
            toggle_inline_code(document);
            true
        }
    }
}

fn handle_set_block_type(document: &web_sys::Document, block_kind: &BlockKind) {
    match block_kind {
        BlockKind::Paragraph => {
            format_block_custom(document, "p", false);
        }
        BlockKind::Heading { level } => {
            let level = (*level).clamp(1, 6);
            let tag = format!("h{}", level);
            format_block_custom(document, &tag, false);
        }
        BlockKind::Quote => {
            format_block_custom(document, "blockquote", true);
        }
        BlockKind::CodeBlock { .. } => {
            format_block_custom(document, "pre", true);
            ensure_pre_has_code(document);
        }
        BlockKind::BulletList { .. } => {
            exec_command(document, "insertUnorderedList");
        }
        BlockKind::OrderedList { .. } => {
            exec_command(document, "insertOrderedList");
        }
        BlockKind::TaskList { .. } => {
            toggle_task_list(document);
        }
        BlockKind::ListItem
        | BlockKind::TaskItem { .. }
        | BlockKind::Image { .. }
        | BlockKind::Embed { .. }
        | BlockKind::Rule
        | BlockKind::Table { .. } => {}
    }
}

/// Custom implementation of formatBlock that works reliably in modern browsers.
/// Replaces the deprecated execCommand('formatBlock') approach.
/// If allow_toggle is true, clicking the same block type again will convert to paragraph.
fn format_block_custom(document: &web_sys::Document, tag_name: &str, allow_toggle: bool) {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    let Some(window) = web_sys::window() else {
        return;
    };

    let Some(selection) = window.get_selection().ok().flatten() else {
        return;
    };

    if selection.range_count() == 0 {
        return;
    }

    let Ok(range) = selection.get_range_at(0) else {
        return;
    };

    // Store the current offset for cursor restoration
    let start_offset = range.start_offset().unwrap_or(0);

    // Get the common ancestor container
    let Ok(container) = range.common_ancestor_container() else {
        return;
    };

    // Find the block-level parent element
    let block_element = find_block_element(&container);

    if let Some(old_block) = block_element {
        // Check if we're already the right type
        if let Some(element) = old_block.dyn_ref::<web_sys::Element>() {
            let current_tag = element.tag_name().to_lowercase();
            if current_tag == tag_name.to_lowercase() {
                // If toggle is allowed, convert to paragraph instead
                if allow_toggle {
                    // Change the tag to "p" to convert to paragraph
                    let Ok(new_element) = document.create_element("p") else {
                        return;
                    };

                    if let Some(html_old) = old_block.dyn_ref::<HtmlElement>() {
                        if let Some(html_new) = new_element.dyn_ref::<HtmlElement>() {
                            html_new.set_inner_html(&html_old.inner_html());

                            if let Some(parent) = old_block.parent_node() {
                                let _ = parent.replace_child(&new_element, &old_block);

                                // Restore cursor
                                let _ = selection.remove_all_ranges();
                                if let Ok(new_range) = document.create_range() {
                                    if let Some(first_child) = new_element.first_child() {
                                        let _ = new_range.set_start(&first_child, start_offset);
                                        let _ = new_range.set_end(&first_child, start_offset);
                                    } else {
                                        let _ = new_range.select_node_contents(&new_element);
                                        let _ = new_range.collapse_with_to_start(false);
                                    }
                                    let _ = selection.add_range(&new_range);
                                }

                                if let Some(html_elem) = new_element.dyn_ref::<HtmlElement>() {
                                    let _ = html_elem.focus();
                                }
                            }
                        }
                    }
                }
                return;
            }
        }

        // Create new element
        let Ok(new_element) = document.create_element(tag_name) else {
            return;
        };

        // Copy innerHTML from old to new
        if let Some(html_old) = old_block.dyn_ref::<HtmlElement>() {
            if let Some(html_new) = new_element.dyn_ref::<HtmlElement>() {
                html_new.set_inner_html(&html_old.inner_html());

                // Copy class attribute if any (preserve styling)
                if let Some(old_elem) = old_block.dyn_ref::<web_sys::Element>() {
                    if let Some(class_attr) = old_elem.get_attribute("class") {
                        let _ = new_element.set_attribute("class", &class_attr);
                    }
                }

                // Replace old with new
                if let Some(parent) = old_block.parent_node() {
                    let _ = parent.replace_child(&new_element, &old_block);

                    // Restore cursor position
                    let _ = selection.remove_all_ranges();
                    if let Ok(new_range) = document.create_range() {
                        // Try to restore the exact cursor position
                        if let Some(first_child) = new_element.first_child() {
                            let _ = new_range.set_start(&first_child, start_offset);
                            let _ = new_range.set_end(&first_child, start_offset);
                        } else {
                            // Fallback: place cursor at the end
                            let _ = new_range.select_node_contents(&new_element);
                            let _ = new_range.collapse_with_to_start(false);
                        }
                        let _ = selection.add_range(&new_range);
                    }

                    // Focus the new element
                    if let Some(html_elem) = new_element.dyn_ref::<HtmlElement>() {
                        let _ = html_elem.focus();
                    }
                }
            }
        }
    } else {
        // No block element found - wrap the selection in a new block
        let Ok(new_element) = document.create_element(tag_name) else {
            return;
        };

        // Try to surround the contents with the new element
        if range.surround_contents(&new_element).is_ok() {
            if let Some(html_new) = new_element.dyn_ref::<HtmlElement>() {
                // Select the content inside the new element
                let _ = selection.remove_all_ranges();
                if let Ok(new_range) = document.create_range() {
                    let _ = new_range.select_node_contents(&new_element);
                    let _ = new_range.collapse_with_to_start(false);
                    let _ = selection.add_range(&new_range);
                }

                // Focus the new element
                let _ = html_new.focus();
            }
        }
    }
}

/// Find the closest block-level element ancestor.
/// Stops at editor-content div to avoid escaping the editor boundary.
fn find_block_element(node: &web_sys::Node) -> Option<web_sys::Node> {
    use wasm_bindgen::JsCast;

    let mut current = Some(node.clone());

    while let Some(node) = current {
        if let Some(element) = node.dyn_ref::<web_sys::Element>() {
            let tag = element.tag_name().to_lowercase();

            // Stop if we hit the editor-content div
            if tag == "div"
                && element
                    .get_attribute("class")
                    .map_or(false, |c| c.contains("editor-content"))
            {
                // If we're at editor-content, return null to create a new paragraph
                return None;
            }

            // Check if it's a formattable block-level element
            if matches!(
                tag.as_str(),
                "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote" | "pre"
            ) {
                return Some(node);
            }
        }
        current = node.parent_node();
    }

    None
}

/// Find the parent block element for reordering purposes.
/// Returns the direct child of the editor-content div.
fn find_parent_block(node: &web_sys::Node, _document: &web_sys::Document) -> Option<web_sys::Element> {
    use wasm_bindgen::JsCast;

    let mut current = Some(node.clone());

    while let Some(node) = current {
        if let Some(element) = node.dyn_ref::<web_sys::Element>() {
            // Check if the parent is the editor-content div
            if let Some(parent) = element.parent_element() {
                if parent.get_attribute("class").map_or(false, |c| c.contains("editor-content")) {
                    // This element is a direct child of editor-content
                    return Some(element.clone());
                }
            }
        }
        current = node.parent_node();
    }

    None
}

fn handle_insert_block(document: &web_sys::Document, block_kind: &BlockKind) {
    match block_kind {
        BlockKind::Rule => {
            exec_command(document, "insertHorizontalRule");
        }
        BlockKind::Image {
            src,
            alt,
            title,
            width,
            height,
            caption,
        } => {
            let html = build_image_html(
                src,
                alt.as_deref(),
                title.as_deref(),
                *width,
                *height,
                caption.as_deref(),
            );
            insert_html(document, &html);
        }
        BlockKind::Embed {
            provider,
            url,
            title,
            width,
            height,
        } => {
            let html = build_embed_html(provider, url, title.as_deref(), *width, *height);
            insert_html(document, &html);
        }
        BlockKind::Table {
            headers,
            rows,
            column_align,
        } => {
            gloo_console::log!("[handle_insert_block] Building table HTML");
            let html = build_table_html(headers, rows, column_align);
            gloo_console::log!("[handle_insert_block] Table HTML:", &html);
            insert_html(document, &html);
            gloo_console::log!("[handle_insert_block] Table HTML inserted");
        }
        BlockKind::Paragraph
        | BlockKind::Heading { .. }
        | BlockKind::BulletList { .. }
        | BlockKind::OrderedList { .. }
        | BlockKind::TaskList { .. }
        | BlockKind::ListItem
        | BlockKind::TaskItem { .. }
        | BlockKind::Quote
        | BlockKind::CodeBlock { .. } => {}
    }
}

fn handle_insert_link(document: &web_sys::Document, command: &InsertLink) {
    let href = command.href.trim();
    if href.is_empty() {
        return;
    }

    exec_command_with_value(document, "createLink", href);

    if let Ok(Some(selection)) = document.get_selection() {
        if let Some(anchor_node) = selection.anchor_node() {
            if let Some(link_element) = find_ancestor_element(&anchor_node, "a") {
                let _ = link_element.set_attribute("href", href);

                if let Some(title) = command
                    .title
                    .as_ref()
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                {
                    let _ = link_element.set_attribute("title", title);
                } else {
                    let _ = link_element.remove_attribute("title");
                }

                if command.target_blank {
                    let _ = link_element.set_attribute("target", "_blank");
                    let _ = link_element.set_attribute("rel", "noopener noreferrer");
                } else {
                    let _ = link_element.remove_attribute("target");
                    let _ = link_element.remove_attribute("rel");
                }
            }
        }
    }
}

fn insert_html(document: &web_sys::Document, html: &str) {
    exec_command_with_value(document, "insertHTML", html);
}

fn toggle_task_list(document: &web_sys::Document) {
    if let Some(list) = selection_list_element(document) {
        if element_has_class(&list, "task-list") {
            convert_to_plain_list(&list);
        } else {
            convert_to_task_list(document, &list);
        }
    } else {
        exec_command(document, "insertUnorderedList");
        if let Some(list) = selection_list_element(document) {
            convert_to_task_list(document, &list);
        }
    }
}

fn toggle_inline_code(document: &web_sys::Document) {
    if selection_in_code(document) {
        remove_inline_code(document);
    } else {
        apply_inline_code(document);
    }
}

fn apply_inline_code(document: &web_sys::Document) {
    if let Ok(Some(selection)) = document.get_selection() {
        let selected_text = selection.to_string().as_string().unwrap_or_default();
        let use_placeholder = selected_text.trim().is_empty();
        let content = if use_placeholder {
            "code".to_string()
        } else {
            selected_text
        };
        let html = format!("<code>{}</code>", escape_html(&content));
        exec_command_with_value(document, "insertHTML", &html);
    }
}

fn remove_inline_code(document: &web_sys::Document) {
    if let Ok(Some(selection)) = document.get_selection() {
        if let Some(anchor) = selection.anchor_node() {
            if let Some(code_element) = find_ancestor_element(&anchor, "code") {
                unwrap_element(code_element);
            }
        }
        if let Some(focus) = selection.focus_node() {
            if let Some(code_element) = find_ancestor_element(&focus, "code") {
                unwrap_element(code_element);
            }
        }
    }
}

fn ensure_pre_has_code(document: &web_sys::Document) {
    if let Ok(Some(selection)) = document.get_selection() {
        if let Some(anchor) = selection.anchor_node() {
            if let Some(pre_element) = find_ancestor_element(&anchor, "pre") {
                if pre_element.query_selector("code").ok().flatten().is_none() {
                    if let Ok(code_element) = document.create_element("code") {
                        while let Some(child) = pre_element.first_child() {
                            let _ = code_element.append_child(&child);
                        }
                        let _ = pre_element.append_child(&code_element);
                    }
                }
            }
        }
    }
}

fn find_ancestor_element(node: &web_sys::Node, tag: &str) -> Option<web_sys::Element> {
    let mut current: Option<web_sys::Node> = Some(node.clone());

    while let Some(node) = current {
        if let Some(element) = node.dyn_ref::<web_sys::Element>() {
            if element.tag_name().eq_ignore_ascii_case(tag) {
                return Some(element.clone());
            }
        }
        current = node.parent_node();
    }

    None
}

fn unwrap_element(element: web_sys::Element) {
    if let Some(parent) = element.parent_node() {
        while let Some(child) = element.first_child() {
            let _ = parent.insert_before(&child, Some(&element));
        }
        let _ = parent.remove_child(&element);
    }
}

pub(crate) fn selection_in_code(document: &web_sys::Document) -> bool {
    if let Ok(Some(selection)) = document.get_selection() {
        if let Some(anchor) = selection.anchor_node() {
            if find_ancestor_element(&anchor, "code").is_some() {
                return true;
            }
        }
        if let Some(focus) = selection.focus_node() {
            if find_ancestor_element(&focus, "code").is_some() {
                return true;
            }
        }
    }
    false
}

fn selection_list_element(document: &web_sys::Document) -> Option<web_sys::Element> {
    if let Ok(Some(selection)) = document.get_selection() {
        if let Some(anchor) = selection.anchor_node() {
            if let Some(list) = find_ancestor_element(&anchor, "ul") {
                return Some(list);
            }
            if let Some(list) = find_ancestor_element(&anchor, "ol") {
                return Some(list);
            }
        }
        if let Some(focus) = selection.focus_node() {
            if let Some(list) = find_ancestor_element(&focus, "ul") {
                return Some(list);
            }
            if let Some(list) = find_ancestor_element(&focus, "ol") {
                return Some(list);
            }
        }
    }
    None
}

fn convert_to_task_list(document: &web_sys::Document, list: &web_sys::Element) {
    add_class(list, "task-list");

    for item in list_items(list) {
        add_class(&item, "task-list-item");
        ensure_checkbox(document, &item);
    }
}

fn convert_to_plain_list(list: &web_sys::Element) {
    remove_class(list, "task-list");

    for item in list_items(list) {
        remove_class(&item, "task-list-item");
        remove_checkbox(&item);
    }
}

fn ensure_checkbox(document: &web_sys::Document, item: &web_sys::Element) {
    if has_checkbox(item) {
        return;
    }

    if let Ok(input) = document.create_element("input") {
        let _ = input.set_attribute("type", "checkbox");
        add_class(&input, "task-checkbox");
        let _ = input.set_attribute("contenteditable", "false");

        let text_node = document.create_text_node(" ");

        if let Some(first_child) = item.first_child() {
            let _ = item.insert_before(&input, Some(&first_child));
            let _ = item.insert_before(&text_node, Some(&first_child));
        } else {
            let _ = item.append_child(&input);
            let _ = item.append_child(&text_node);
        }
    }
}

fn remove_checkbox(item: &web_sys::Element) {
    if let Some(first_element) = item.first_element_child() {
        if first_element.tag_name().eq_ignore_ascii_case("input") {
            let _ = item.remove_child(&first_element);
        }
    }

    if let Some(first_child) = item.first_child() {
        if first_child.node_type() == web_sys::Node::TEXT_NODE {
            if first_child
                .node_value()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                let _ = item.remove_child(&first_child);
            }
        }
    }
}

fn has_checkbox(item: &web_sys::Element) -> bool {
    if let Some(first_element) = item.first_element_child() {
        return first_element.tag_name().eq_ignore_ascii_case("input");
    }
    false
}

fn list_items(list: &web_sys::Element) -> Vec<web_sys::Element> {
    if let Ok(nodes) = list.query_selector_all(":scope > li") {
        let mut items = Vec::with_capacity(nodes.length() as usize);
        for idx in 0..nodes.length() {
            if let Some(node) = nodes.item(idx) {
                if let Ok(element) = node.dyn_into::<web_sys::Element>() {
                    items.push(element);
                }
            }
        }
        items
    } else {
        Vec::new()
    }
}

fn add_class(element: &web_sys::Element, class_name: &str) {
    if element_has_class(element, class_name) {
        return;
    }

    let mut classes: Vec<String> = element
        .get_attribute("class")
        .unwrap_or_default()
        .split_whitespace()
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect();
    classes.push(class_name.to_string());
    let _ = element.set_attribute("class", &classes.join(" "));
}

fn remove_class(element: &web_sys::Element, class_name: &str) {
    let mut classes: Vec<String> = element
        .get_attribute("class")
        .unwrap_or_default()
        .split_whitespace()
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case(class_name))
        .map(|value| value.to_string())
        .collect();

    if classes.is_empty() {
        let _ = element.remove_attribute("class");
    } else {
        classes.dedup();
        let _ = element.set_attribute("class", &classes.join(" "));
    }
}

fn element_has_class(element: &web_sys::Element, class_name: &str) -> bool {
    element
        .get_attribute("class")
        .map(|value| {
            value
                .split_whitespace()
                .any(|existing| existing.eq_ignore_ascii_case(class_name))
        })
        .unwrap_or(false)
}

fn sync_task_checkbox_state(document: &web_sys::Document) {
    if let Ok(node_list) = document.query_selector_all(".task-checkbox") {
        for idx in 0..node_list.length() {
            if let Some(node) = node_list.item(idx) {
                if let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>() {
                    if input.checked() {
                        let _ = input.set_attribute("checked", "checked");
                    } else {
                        let _ = input.remove_attribute("checked");
                    }
                }
            }
        }
    }
}

fn build_image_html(
    src: &str,
    alt: Option<&str>,
    title: Option<&str>,
    width: Option<u32>,
    height: Option<u32>,
    caption: Option<&str>,
) -> String {
    let src_attr = escape_attribute(src);
    let alt_attr = escape_attribute(alt.unwrap_or(""));
    let title_attr = title
        .map(|value| format!(" title=\"{}\"", escape_attribute(value)))
        .unwrap_or_default();
    let width_attr = width
        .map(|value| format!(" width=\"{}\"", value))
        .unwrap_or_default();
    let height_attr = height
        .map(|value| format!(" height=\"{}\"", value))
        .unwrap_or_default();

    let img_tag = format!(
        "<img src=\"{}\" alt=\"{}\"{}{}{} class=\"editor-image max-w-full rounded-md\" loading=\"lazy\" />",
        src_attr, alt_attr, title_attr, width_attr, height_attr
    );

    if let Some(caption_text) = caption.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(escape_html(trimmed))
        }
    }) {
        format!(
            "<figure class=\"editor-figure text-center\">{}<figcaption class=\"mt-2 text-sm text-gray-500 dark:text-gray-400\">{}</figcaption></figure>",
            img_tag, caption_text
        )
    } else {
        img_tag
    }
}

fn build_embed_html(
    provider: &EmbedProvider,
    url: &str,
    title: Option<&str>,
    width: Option<u32>,
    height: Option<u32>,
) -> String {
    let src_attr = escape_attribute(url);
    let title_attr = escape_attribute(title.unwrap_or(match provider {
        EmbedProvider::Youtube => "YouTube embed",
        EmbedProvider::X => "X embed",
        EmbedProvider::Generic => "Embedded content",
    }));
    let width_attr = width
        .map(|value| format!(" width=\"{}\"", value))
        .unwrap_or_default();
    let height_attr = height
        .map(|value| format!(" height=\"{}\"", value))
        .unwrap_or_default();

    let allow_attr = match provider {
        EmbedProvider::Youtube => "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share",
        EmbedProvider::X => "clipboard-write",
        EmbedProvider::Generic => "accelerometer; autoplay; encrypted-media",
    };

    format!(
        "<div class=\"editor-embed aspect-video\"><iframe src=\"{}\" title=\"{}\" frameborder=\"0\" allow=\"{}\" allowfullscreen{}{}></iframe></div>",
        src_attr, title_attr, allow_attr, width_attr, height_attr
    )
}

fn build_table_html(
    headers: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    column_align: &[ast::TableAlign],
) -> String {
    use super::renderer::render_inlines;

    let mut html = String::from("<div class=\"mb-4 overflow-x-auto\"><table class=\"min-w-full border-collapse border border-gray-300 dark:border-gray-600\">");

    // Render headers
    if !headers.is_empty() {
        html.push_str("<thead class=\"bg-gray-100 dark:bg-gray-700\"><tr>");
        for (i, header_cell) in headers.iter().enumerate() {
            let align = column_align.get(i).unwrap_or(&ast::TableAlign::Left);
            let align_class = match align {
                ast::TableAlign::Left => " text-left",
                ast::TableAlign::Center => " text-center",
                ast::TableAlign::Right => " text-right",
            };
            let content = render_inlines(header_cell);
            html.push_str(&format!(
                "<th class=\"border border-gray-300 dark:border-gray-600 px-4 py-2 font-semibold text-gray-900 dark:text-gray-100{}\">{}</th>",
                align_class, content
            ));
        }
        html.push_str("</tr></thead>");
    }

    // Render rows
    if !rows.is_empty() {
        html.push_str("<tbody>");
        for row in rows {
            html.push_str("<tr>");
            for (i, cell) in row.iter().enumerate() {
                let align = column_align.get(i).unwrap_or(&ast::TableAlign::Left);
                let align_class = match align {
                    ast::TableAlign::Left => " text-left",
                    ast::TableAlign::Center => " text-center",
                    ast::TableAlign::Right => " text-right",
                };
                let content = render_inlines(cell);
                html.push_str(&format!(
                    "<td class=\"border border-gray-300 dark:border-gray-600 px-4 py-2 text-gray-900 dark:text-gray-100{}\">{}</td>",
                    align_class, content
                ));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody>");
    }

    html.push_str("</table></div>");
    html
}

fn escape_attribute(value: &str) -> String {
    escape_html(value)
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Read-only viewer for rendered content.
#[component]
pub fn ContentViewer(value: String, #[props(default = String::new())] class: String) -> Element {
    let doc = use_memo(move || serde_json::from_str::<Doc>(&value).unwrap_or_default());

    let html_content = use_memo(move || render_doc(&doc.read()));

    rsx! {
        div {
            class: "content-viewer prose dark:prose-invert max-w-none {class}",
            style: "white-space: normal; overflow-wrap: break-word;",
            dangerous_inner_html: "{html_content}",
        }
    }
}

/// Strip HTML tags for character counting.
#[allow(dead_code)]
fn strip_html_tags(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(html, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_strips_html_tags() {
        let html = "<p>Hello <strong>world</strong></p>";
        let stripped = strip_html_tags(html);
        assert_eq!(stripped, "Hello world");
    }

    #[test]
    fn it_parses_initial_value() {
        let json = r#"{"blocks":[{"id":"test","kind":{"type":"paragraph"},"align":"left","attrs":{},"children":[{"type":"text","text":"Test","marks":{}}]}]}"#;
        let doc: Doc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.blocks.len(), 1);
    }

    #[test]
    fn it_renders_empty_content() {
        let doc = Doc::default();
        let html = render_doc(&doc);
        // Should have at least the default paragraph
        assert!(!html.is_empty());
    }
}
