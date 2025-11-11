use dioxus::{logger::tracing, prelude::*};
use futures_util::StreamExt;

use super::form::{use_blog_form, BlogForm};
// use crate::components::editor::RichTextEditor; // Moved to legacy - using TypeScript editor instead
use crate::components::{
    AppInput, ConfirmDialog, EditorJsHost, ImageEditorModal, MediaUploadItem, MediaUploadZone,
    PostSuccessDialog,
};
use crate::hooks::use_previous;
use crate::router::Route;
use crate::store::{
    use_categories, use_image_editor, use_media, use_post, use_tag, MediaReference,
    MediaUploadPayload, PostContent, PostCreatePayload, PostEditPayload, PostStatus,
};
use crate::ui::shadcn::{
    Badge, BadgeVariant, Button, ButtonVariant, Checkbox, Combobox, ComboboxItem, Skeleton,
};
use serde_json;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Blob, Url};

#[component]
pub fn BlogFormContainer(post_id: Option<i32>) -> Element {
    let posts = use_post();
    let categories = use_categories();
    let tags = use_tag();
    let nav = use_navigator();
    let media_state = use_media();
    let editor_state = use_image_editor();

    // Image editing state
    let mut edit_confirm_open = use_signal(|| false);
    let mut pending_file = use_signal(|| None::<web_sys::File>);
    let mut pending_field = use_signal(|| None::<String>);

    // Success dialog state
    let mut success_dialog_open = use_signal(|| false);

    // Initialize form with existing post data if editing
    let mut initial_form = use_signal(|| None::<BlogForm>);
    let is_edit_mode = post_id.is_some();

    // Fetch existing post if editing
    use_effect(move || {
        if let Some(id) = post_id {
            spawn(async move {
                posts.view_by_id(id).await;
            });
        }
    });

    // Fetch categories and tags
    use_effect(move || {
        spawn(async move {
            categories.list().await;
            tags.list().await;
        });
    });

    // Populate form when post data is loaded
    use_effect(move || {
        if let Some(id) = post_id {
            let view_frame = posts.view.read();
            if let Some(post_frame) = view_frame.get(&id) {
                if let Some(post) = &post_frame.data {
                    let form = BlogForm {
                        title: post.title.clone(),
                        content: serde_json::to_string(&post.content).unwrap_or_default(),
                        slug: post.slug.clone(),
                        excerpt: post.excerpt.clone().unwrap_or_default(),
                        featured_image_blob_url: post
                            .featured_image
                            .as_ref()
                            .map(|m| m.file_url.clone()),
                        featured_image_media_id: post.featured_image.as_ref().map(|m| m.id),
                        is_published: post.status == PostStatus::Published,
                        category_id: Some(post.category.id),
                        tag_ids: post.tags.iter().map(|t| t.id).collect(),
                    };
                    initial_form.set(Some(form));
                }
            }
        } else {
            // New post: prefill content from localStorage draft if present
            let mut form = BlogForm::new();
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(draft)) = storage.get_item("blog_form_draft_content") {
                        form.content = draft;
                    }
                }
            }
            initial_form.set(Some(form));
        }
    });

    // Wait for form to be initialized
    let form_data = initial_form.read();
    if form_data.is_none() {
        return rsx! {
            div { class: "flex items-center justify-center min-h-[400px]",
                div { class: "text-center",
                    div { class: "inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]" }
                    p { class: "mt-4 text-sm text-muted-foreground", "Loading..." }
                }
            }
        };
    }

    let blog_form_hook = use_blog_form(form_data.clone().unwrap());
    let mut form = blog_form_hook.form;
    let mut auto_slug = blog_form_hook.auto_slug;
    // Use coroutine to handle editor changes from JavaScript
    let editor_change_handler = {
        let form_signal = form;

        use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
            let mut form_signal = form_signal;

            while let Some(detail) = rx.next().await {
                form_signal.write().update_field("content", detail.clone());

                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let _ = storage.set_item("blog_form_draft_content", &detail);
                    }
                }
            }
        })
    };

    let mut listener_handle =
        use_signal(|| None::<(web_sys::EventTarget, Closure<dyn FnMut(web_sys::Event)>)>);

    use_effect(move || {
        if let Some((event_target, listener)) = listener_handle.write().take() {
            let _ = event_target.remove_event_listener_with_callback(
                "editor:change",
                listener.as_ref().unchecked_ref(),
            );
        }

        if let Some(window) = web_sys::window() {
            let event_target: web_sys::EventTarget = window.clone().into();
            let handler = editor_change_handler.clone();
            let listener = Closure::wrap(Box::new(move |event: web_sys::Event| {
                if let Ok(custom_event) = event.dyn_into::<web_sys::CustomEvent>() {
                    if let Some(detail) = custom_event.detail().as_string() {
                        // Send to coroutine which runs in Dioxus context
                        handler.send(detail);
                    }
                }
            }) as Box<dyn FnMut(_)>);

            if event_target
                .add_event_listener_with_callback(
                    "editor:change",
                    listener.as_ref().unchecked_ref(),
                )
                .is_ok()
            {
                listener_handle.set(Some((event_target, listener)));
            } else {
                listener.forget();
            }
        }
    });

    // Track upload status and resolve media IDs
    use_effect(move || {
        let form_data = form.read().data.clone();

        // Check featured image blob URL
        if let Some(featured_blob) = &form_data.featured_image_blob_url {
            if form_data.featured_image_media_id.is_none() {
                // Check if upload completed
                if let Some(media) = media_state.get_uploaded_media(featured_blob) {
                    gloo_console::log!(
                        "[BlogForm] Featured image upload complete, media ID:",
                        media.id.to_string()
                    );
                    let mut form_mut = form.write();
                    form_mut.data.featured_image_media_id = Some(media.id);
                }
            }
        }
    });

    // Handle file selection from upload zone
    let handle_file_selected = move |_field_name: String| {
        move |files: Vec<web_sys::File>| {
            if let Some(file) = files.first() {
                pending_file.set(Some(file.clone()));
                pending_field.set(Some(_field_name.clone()));
                edit_confirm_open.set(true);
            }
        }
    };

    // Handle edit confirmation - open image editor
    let handle_edit_confirm = move |_| {
        let file = pending_file();
        if let Some(f) = file {
            gloo_console::log!("[BlogForm] Opening editor for file:", f.name());
            // Create blob URL for the file
            let blob: &Blob = f.as_ref();
            if let Ok(blob_url) = Url::create_object_url_with_blob(blob) {
                spawn(async move {
                    let _ = editor_state.open_editor(Some(f.clone()), blob_url).await;
                });
            }
        }
    };

    // Handle edit skip - upload directly
    let handle_edit_skip = move |_| {
        let file = pending_file();
        let field = pending_field();

        if let (Some(f), Some(_field_name)) = (file, field) {
            gloo_console::log!("[BlogForm] Skipping edit, uploading directly:", f.name());

            // Upload the file
            spawn(async move {
                let payload = MediaUploadPayload {
                    file: f.clone(),
                    reference_type: Some(MediaReference::Post),
                    width: None,
                    height: None,
                };

                match media_state.upload(payload).await {
                    Ok(blob_url) => {
                        gloo_console::log!("[BlogForm] Upload successful:", &blob_url);
                        let mut form_mut = form.write();
                        form_mut.data.featured_image_blob_url = Some(blob_url);
                    }
                    Err(e) => {
                        gloo_console::error!("[BlogForm] Upload failed:", e);
                    }
                }
            });
        }
    };

    // Handle image editor save - upload edited file
    let handle_editor_save = move |edited_file: web_sys::File| {
        let field = pending_field();

        if let Some(_field_name) = field {
            gloo_console::log!(
                "[BlogForm] Editor saved, uploading edited file:",
                edited_file.name()
            );

            // Upload the edited file
            spawn(async move {
                let payload = MediaUploadPayload {
                    file: edited_file,
                    reference_type: Some(MediaReference::Post),
                    width: None,
                    height: None,
                };

                match media_state.upload(payload).await {
                    Ok(blob_url) => {
                        gloo_console::log!("[BlogForm] Edited upload successful:", &blob_url);
                        let mut form_mut = form.write();
                        form_mut.data.featured_image_blob_url = Some(blob_url);
                    }
                    Err(e) => {
                        gloo_console::error!("[BlogForm] Edited upload failed:", e);
                    }
                }
            });
        }
    };

    // Handle re-edit of already uploaded image
    let handle_edit_uploaded = move |_field: String| {
        move |blob_url: String| {
            gloo_console::log!("[BlogForm] Re-editing uploaded image:", &blob_url);
            pending_field.set(Some(_field.clone()));
            // Open editor directly with the blob URL
            spawn(async move {
                let _ = editor_state.open_editor(None, blob_url).await;
            });
        }
    };

    // Handle successful submission with use_previous to detect state transitions
    let add_state = posts.add.read();
    let edit_state = if let Some(id) = post_id {
        posts.edit.read().get(&id).cloned()
    } else {
        None
    };
    let any_success = (post_id.is_none() && add_state.is_success())
        || (post_id.is_some() && edit_state.as_ref().map_or(false, |s| s.is_success()));

    let prev_success = use_previous(any_success);

    // Compute post_id reactively for the dialog
    let dialog_post_id = use_memo(move || {
        if post_id.is_some() {
            post_id
        } else {
            posts.add.read().data.as_ref().map(|d| d.id)
        }
    });

    tracing::debug!(
        "[BlogForm] dialog_postid: {:?} add_success: {}, any_success: {}, prev_success: {:?}",
        *dialog_post_id.read(),
        add_state.is_success(),
        any_success,
        prev_success
    );

    use_effect(use_reactive!(|(any_success,)| {
        if let Some(prev) = prev_success {
            if !prev && any_success {
                // Success state just transitioned from false to true
                // Clean up localStorage draft
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let _ = storage.remove_item("blog_form_draft_content");
                    }
                }

                // Open success dialog
                success_dialog_open.set(true);
            }
        }
    }));

    rsx! {
        div {
            h1 { class: "sr-only",
                if is_edit_mode { "Edit Blog Post" } else { "New Blog Post" }
            }

            // Show error message if submission failed
            {
                let add_state = posts.add.read();
                let edit_state = if let Some(id) = post_id {
                    posts.edit.read().get(&id).cloned()
                } else {
                    None
                };

                let is_failed = add_state.is_failed() || edit_state.as_ref().map_or(false, |s| s.is_failed());
                let error_message = add_state.error_message().or_else(|| edit_state.as_ref().and_then(|s| s.error_message()));

                if is_failed {
                    rsx! {
                        div { class: "rounded-md border border-red-200 bg-red-50 p-4 mb-6 text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-300",
                            div { class: "flex items-start gap-3",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    class: "h-5 w-5 flex-shrink-0",
                                    view_box: "0 0 20 20",
                                    fill: "currentColor",
                                    path {
                                        fill_rule: "evenodd",
                                        d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z",
                                        clip_rule: "evenodd"
                                    }
                                }
                                span { class: "text-sm font-medium",
                                    {error_message.unwrap_or_else(|| "Failed to save post".to_string())}
                                }
                            }
                        }
                    }
                } else {
                    rsx! { }
                }
            }

            div { class: "grid grid-cols-1 gap-10 lg:grid-cols-3",
                // Main column
                div { class: "lg:col-span-2 space-y-8",
                    // Post details card
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 py-6",
                            h2 { class: "text-lg font-semibold", "Post Content" }
                            p { class: "text-sm text-muted-foreground", "Write and format your blog post content." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            form {
                                class: "space-y-6",
                                onsubmit: move |e| {
                                    e.prevent_default();
                                },

                    AppInput {
                        name: "title",
                        form,
                        label: "Title",
                        placeholder: "Post title",
                        onblur: move |_| {
                            if *auto_slug.read() && !is_edit_mode {
                                let title_value = form.peek().get_field("title").unwrap().value.clone();
                                let sanitized = BlogForm::sanitize_slug(&title_value);
                                form.write().update_field("slug", sanitized);
                            }
                        },
                    }
                    div { class: "space-y-2",
                        div { class: "flex justify-between items-center",
                            label { class: "block text-sm font-medium text-foreground", "Slug" }
                            if !is_edit_mode {
                                label { class: "flex items-center gap-2 cursor-pointer",
                                    span { class: "text-sm text-muted-foreground", "Auto-generate" }
                                    Checkbox {
                                        checked: *auto_slug.read(),
                                        onchange: move |checked: bool| {
                                            auto_slug.set(checked);
                                        },
                                    }
                                }
                            }
                        }
                        AppInput {
                            form,
                            name: "slug",
                            r#type: "text",
                            readonly: *auto_slug.read() && !is_edit_mode,
                            placeholder: "Post slug",
                        }
                    }

                    div { class: "space-y-2",
                        label { class: "block text-sm font-medium text-foreground", "Excerpt" }
                        textarea {
                            class: "w-full h-24 resize-none rounded-md border border-border/70 bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-muted-foreground transition-colors duration-200 focus:border-ring focus:ring-2 focus:ring-ring/40",
                            placeholder: "Brief description of the post",
                            rows: "3",
                            value: form.read().data.excerpt.clone(),
                            oninput: move |event| {
                                form.write().update_field("excerpt", event.value());
                            },
                        }
                    }

                    div { class: "h-px bg-border/60" }

                    {
                        let content_value = {
                            let reader = form.read();
                            reader.data.content.clone()
                        };

                        let initial_json = if content_value.trim().is_empty() {
                            None
                        } else {
                            Some(content_value)
                        };

                        rsx! {
                            div { class: "space-y-2",
                                label { class: "block text-sm font-medium text-foreground",
                                    "Content "
                                    span { class: "text-red-500", "*" }
                                }
                                EditorJsHost { initial_json }
                            }
                        }
                    }
                            }
                        }
                    }
                }

                // Sidebar column
                div { class: "space-y-8",
                    // Metadata card
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Metadata" }
                            p { class: "text-sm text-muted-foreground", "Organize and categorize your post." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                    // Published status switch
                    div { class: "flex items-center justify-between",
                        div { class: "space-y-0.5",
                            label { class: "block text-sm font-medium text-foreground", "Published" }
                            p { class: "text-xs text-muted-foreground",
                                if form.read().data.is_published { "This post is publicly visible." } else { "This post is saved as draft." }
                            }
                        }
                        input {
                            class: "relative h-6 w-11 cursor-pointer appearance-none rounded-full bg-border/50 transition-colors duration-200 checked:bg-primary focus:ring-2 focus:ring-ring/40 before:pointer-events-none before:absolute before:h-5 before:w-5 before:translate-x-0.5 before:translate-y-0.5 before:rounded-full before:bg-white before:shadow-sm before:transition-transform before:duration-200 checked:before:translate-x-5",
                            r#type: "checkbox",
                            checked: form.read().data.is_published,
                            onchange: move |event| {
                                form.write().data.is_published = event.checked();
                            },
                        }
                    }

                    div { class: "h-px bg-border/60" }

                    // Category selection
                    div { class: "space-y-2",
                        label { class: "block text-sm font-medium text-foreground",
                            "Category "
                            span { class: "text-red-500", "*" }
                        }
                        {
                            let cat_list = categories.list.read();
                            let is_loading = cat_list.is_loading();
                            let is_failed = cat_list.is_failed();
                            let items: Vec<ComboboxItem> = cat_list
                                .data
                                .as_ref()
                                .map(|d| &d.data)
                                .unwrap_or(&vec![])
                                .iter()
                                .map(|c| ComboboxItem { value: c.id.to_string(), label: c.name.clone() })
                                .collect();
                            let current_val = form.read().data.category_id.map(|id| id.to_string());

                            rsx! {
                                if is_loading && items.is_empty() {
                                    Skeleton { class: Some("h-10 w-full".to_string()) }
                                } else if is_failed {
                                    div { class: "rounded-md border border-red-200 bg-red-50 p-3 text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-300",
                                        p { class: "text-sm", "Failed to load categories" }
                                    }
                                } else {
                                    Combobox {
                                        items,
                                        placeholder: "Select category...".to_string(),
                                        value: current_val,
                                        width: "w-full".to_string(),
                                        onvaluechange: Some(EventHandler::new(move |val: Option<String>| {
                                            if let Some(v) = val {
                                                if let Ok(id) = v.parse::<i32>() {
                                                    form.write().data.category_id = Some(id);
                                                }
                                            }
                                        })),
                                    }
                                }
                            }
                        }
                    }

                    // Tags selection
                    div { class: "space-y-3",
                        label { class: "block text-sm font-medium text-foreground", "Tags" }
                        p { class: "text-xs text-muted-foreground", "Select tags that categorize your post" }

                        {
                            let tags_list = tags.list.read();
                            tags_list.data.as_ref().map(|tags_data| {
                                let selected_tags: Vec<_> = tags_data.data.iter()
                                    .filter(|tag| form.read().data.tag_ids.contains(&tag.id))
                                    .collect();

                                rsx! {
                                    // Selected tags display
                                    if !selected_tags.is_empty() {
                                        div { class: "flex flex-wrap gap-2 p-3 border border-border/70 rounded-lg bg-muted/30",
                                            for tag in selected_tags {
                                                {
                                                    let tag_id = tag.id;
                                                    let tag_name = tag.name.clone();
                                                    rsx! {
                                                        Badge {
                                                            key: "{tag_id}",
                                                            variant: BadgeVariant::Secondary,
                                                            class: "cursor-pointer hover:bg-destructive hover:text-white transition-colors",
                                                            onclick: move |_| {
                                                                form.write().data.tag_ids.retain(|&id| id != tag_id);
                                                            },
                                                            "{tag_name}"
                                                            span { class: "ml-1", "×" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Available tags
                                    div { class: "flex flex-wrap gap-2 p-4 border border-border/70 rounded-lg bg-transparent",
                                        for tag in &tags_data.data {
                                            {
                                                let tag_id = tag.id;
                                                let tag_name = tag.name.clone();
                                                let is_selected = form.read().data.tag_ids.contains(&tag_id);

                                                if !is_selected {
                                                    rsx! {
                                                        Badge {
                                                            key: "{tag_id}",
                                                            variant: BadgeVariant::Outline,
                                                            class: "cursor-pointer hover:bg-primary hover:text-primary-foreground transition-colors",
                                                            onclick: move |_| {
                                                                form.write().data.tag_ids.push(tag_id);
                                                            },
                                                            "+ {tag_name}"
                                                        }
                                                    }
                                                } else {
                                                    rsx! { }
                                                }
                                            }
                                        }
                                    }
                                }
                            }).unwrap_or_else(|| rsx! {
                                div { class: "text-sm text-muted-foreground p-4 border border-border/70 rounded-lg", "Loading tags..." }
                            })
                        }
                    }
                        }
                    }

                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Featured Image" }
                            p { class: "text-sm text-muted-foreground", "Main image displayed with your post." }
                        }
                        div { class: "px-6 py-6",
                    {
                            let form_data = form.read();
                            let has_featured_blob = form_data.data.featured_image_blob_url.is_some();
                            let featured_blob_url = form_data.data.featured_image_blob_url.clone();

                            rsx! {
                                if has_featured_blob {
                                    // Show uploaded image with status
                                    {
                                        let blob = featured_blob_url.as_ref().unwrap();
                                        let file_info = media_state.get_file_info(blob);
                                        let (filename, file_size) = if let Some(info) = file_info {
                                            (info.filename, info.size)
                                        } else {
                                            ("Featured Image".to_string(), 0)
                                        };

                                        rsx! {
                                            MediaUploadItem {
                                                blob_url: blob.clone(),
                                                filename,
                                                file_size,
                                                on_remove: move |_url: String| {
                                                    let mut form_mut = form.write();
                                                    form_mut.data.featured_image_blob_url = None;
                                                    form_mut.data.featured_image_media_id = None;
                                                },
                                                on_edit: Some(EventHandler::new(handle_edit_uploaded("featured_image".to_string()))),
                                            }
                                        }
                                    }
                                } else {
                                    MediaUploadZone {
                                        on_upload: move |_blob_urls: Vec<String>| {
                                            // Not used - we use on_file_selected instead
                                        },
                                        on_file_selected: Some(EventHandler::new(handle_file_selected("featured_image".to_string()))),
                                        reference_type: Some(MediaReference::Post),
                                        max_files: 1,
                                        allowed_types: vec!["image/".to_string()],
                                        title: "Upload featured image".to_string(),
                                        description: "Click to select an image file".to_string(),
                                        multiple: false,
                                    }
                                }
                            }
                        }
                    }
                        }

                }
            }

            // Form actions
            div { class: "flex justify-end gap-4",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| {
                        nav.push(Route::PostsListScreen {});
                    },
                    "Cancel"
                }
                Button {
                    disabled: {
                        let add_state = posts.add.read();
                        let edit_state = if let Some(id) = post_id {
                            posts.edit.read().get(&id).cloned()
                        } else {
                            None
                        };
                        add_state.is_loading() || edit_state.as_ref().map_or(false, |s| s.is_loading())
                    },
                    onclick: move |_| {
                        let form_data = form.read();

                        // Validate required fields
                        if form_data.data.title.is_empty() {
                            tracing::error!("Title is required");
                            return;
                        }
                        if form_data.data.content.is_empty() {
                            tracing::error!("Content is required");
                            return;
                        }
                        if form_data.data.slug.is_empty() {
                            tracing::error!("Slug is required");
                            return;
                        }
                        if form_data.data.category_id.is_none() {
                            tracing::error!("Category is required");
                            return;
                        }

                        if let Some(id) = post_id {
                            // Edit existing post
                            let payload = PostEditPayload {
                                        title: Some(form_data.data.title.clone()),
                                        content: Some(serde_json::from_str::<PostContent>(&form_data.data.content).unwrap()),
                                        slug: Some(form_data.data.slug.clone()),
                                        excerpt: if form_data.data.excerpt.is_empty() {
                                            None
                                        } else {
                                            Some(form_data.data.excerpt.clone())
                                        },
                                        featured_image_id: form_data.data.featured_image_media_id,
                                        status: Some(if form_data.data.is_published {
                                            PostStatus::Published
                                        } else {
                                            PostStatus::Draft
                                        }),
                                        category_id: form_data.data.category_id,
                                        tag_ids: Some(form_data.data.tag_ids.clone()),
                                        published_at: None,
                                    };

                                    spawn(async move {
                                        posts.edit(id, payload).await;
                                    });
                                } else {
                                    // Create new post
                                    let payload = PostCreatePayload {
                                        title: form_data.data.title.clone(),
                                        content: serde_json::from_str::<PostContent>(&form_data.data.content).unwrap(),
                                        slug: form_data.data.slug.clone(),
                                        excerpt: if form_data.data.excerpt.is_empty() {
                                            None
                                        } else {
                                            Some(form_data.data.excerpt.clone())
                                        },
                                        featured_image_id: form_data.data.featured_image_media_id,
                                        is_published: form_data.data.is_published,
                                        category_id: form_data.data.category_id.unwrap(),
                                        tag_ids: form_data.data.tag_ids.clone(),
                                        published_at: None,
                                    };

                                    spawn(async move {
                                        posts.add(payload).await;
                                    });
                                }
                            },
                            {
                                let add_state = posts.add.read();
                                let edit_state = if let Some(id) = post_id {
                                    posts.edit.read().get(&id).cloned()
                                } else {
                                    None
                                };
                                let is_submitting = add_state.is_loading() || edit_state.as_ref().map_or(false, |s| s.is_loading());

                                if is_submitting {
                                    rsx! {
                                        span { class: "mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-solid border-current border-r-transparent" }
                                    }
                                } else {
                                    rsx! { }
                                }
                            }
                            if is_edit_mode { "Update Post" } else { "Create Post" }
                }
            }

            // Confirm edit dialog
            ConfirmDialog {
                is_open: edit_confirm_open,
                title: "Edit image before uploading?".to_string(),
                description: "You can crop, resize, rotate, or compress the image before uploading.".to_string(),
                confirm_label: "Edit Image".to_string(),
                cancel_label: "Skip & Upload".to_string(),
                on_confirm: handle_edit_confirm,
                on_cancel: handle_edit_skip,
            }

            // Image editor modal
            ImageEditorModal {
                on_save: handle_editor_save,
            }

            PostSuccessDialog {
                is_open: success_dialog_open,
                post_id: dialog_post_id,
                is_new_post: post_id.is_none(),
            }
        }
    }
}
