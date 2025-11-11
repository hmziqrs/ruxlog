use dioxus::prelude::*;

use super::form::{use_categories_form, CategoryForm};
use crate::components::{
    AppInput, ColorPicker, ConfirmDialog, ImageEditorModal, MediaUploadItem, MediaUploadZone,
};
use crate::hooks::OxForm;
use crate::router::Route;
use crate::store::{
    use_categories, use_image_editor, use_media, MediaReference, MediaUploadPayload,
};
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{
    Button, ButtonSize, ButtonVariant, Checkbox, Combobox, ComboboxItem, Skeleton,
};
use hmziq_dioxus_free_icons::{icons::ld_icons::LdLoader, Icon};
use web_sys::{Blob, Url};

#[derive(Props, PartialEq, Clone)]
pub struct CategoryFormContainerProps {
    #[props(default)]
    pub initial: Option<CategoryForm>,
    pub on_submit: EventHandler<CategoryForm>,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub submit_label: Option<String>,
}

#[component]
pub fn CategoryFormContainer(props: CategoryFormContainerProps) -> Element {
    let nav = use_navigator();
    let initial_category_form = props.initial.clone().unwrap_or_else(CategoryForm::new);
    let reset_template = initial_category_form.clone();
    let category_form_hook = use_categories_form(initial_category_form);
    let mut form = category_form_hook.form;
    let mut reset_dialog_open = use_signal(|| false);
    let is_form_dirty = form.read().is_dirty();
    let cats_state = use_categories();
    let media_state = use_media();
    let editor_state = use_image_editor();

    // Image editing state
    let mut pending_file = use_signal(|| None::<web_sys::File>);
    let mut pending_field = use_signal(|| None::<String>); // "logo" or "cover"
    let mut edit_confirm_open = use_signal(|| false);

    // Fetch categories for parent selection on mount
    use_effect(move || {
        spawn(async move {
            cats_state.list().await;
        });
    });

    // Track upload status and resolve media IDs
    use_effect(move || {
        let form_data = form.read().data.clone();

        // Check logo blob URL
        if let Some(logo_blob) = &form_data.logo_blob_url {
            if form_data.logo_media_id.is_none() {
                // Check if upload completed
                if let Some(media) = media_state.get_uploaded_media(logo_blob) {
                    gloo_console::log!(
                        "[CategoryForm] Logo upload complete, media ID:",
                        media.id.to_string()
                    );
                    let mut form_mut = form.write();
                    form_mut.data.logo_media_id = Some(media.id);
                }
            }
        }

        // Check cover blob URL
        if let Some(cover_blob) = &form_data.cover_blob_url {
            if form_data.cover_media_id.is_none() {
                // Check if upload completed
                if let Some(media) = media_state.get_uploaded_media(cover_blob) {
                    gloo_console::log!(
                        "[CategoryForm] Cover upload complete, media ID:",
                        media.id.to_string()
                    );
                    let mut form_mut = form.write();
                    form_mut.data.cover_media_id = Some(media.id);
                }
            }
        }
    });

    // Handle file selection - show edit confirmation dialog
    let handle_file_selected = move |field: String| {
        move |files: Vec<web_sys::File>| {
            if let Some(file) = files.first() {
                gloo_console::log!("[CategoryForm] File selected for", &field, ":", file.name());
                pending_file.set(Some(file.clone()));
                pending_field.set(Some(field.clone()));
                edit_confirm_open.set(true);
            }
        }
    };

    // Handle edit confirmation - open image editor
    let handle_edit_confirm = move |_| {
        let file = pending_file();
        if let Some(f) = file {
            gloo_console::log!("[CategoryForm] Opening editor for file:", f.name());
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

        if let (Some(f), Some(field_name)) = (file, field) {
            gloo_console::log!(
                "[CategoryForm] Skipping edit, uploading directly:",
                f.name()
            );

            // Upload the file
            spawn(async move {
                let payload = MediaUploadPayload {
                    file: f.clone(),
                    reference_type: Some(MediaReference::Category),
                    width: None,
                    height: None,
                };

                match media_state.upload(payload).await {
                    Ok(blob_url) => {
                        gloo_console::log!("[CategoryForm] Upload successful:", &blob_url);
                        let mut form_mut = form.write();
                        if field_name == "logo" {
                            form_mut.data.logo_blob_url = Some(blob_url);
                            // Clear existing media info to use upload tracking
                            form_mut.data.logo_filename = None;
                            form_mut.data.logo_size = None;
                        } else if field_name == "cover" {
                            form_mut.data.cover_blob_url = Some(blob_url);
                            // Clear existing media info to use upload tracking
                            form_mut.data.cover_filename = None;
                            form_mut.data.cover_size = None;
                        }
                    }
                    Err(e) => {
                        gloo_console::error!("[CategoryForm] Upload failed:", e);
                    }
                }
            });
        }
    };

    // Handle image editor save - upload edited file
    let handle_editor_save = move |edited_file: web_sys::File| {
        let field = pending_field();

        if let Some(field_name) = field {
            gloo_console::log!(
                "[CategoryForm] Editor saved, uploading edited file:",
                edited_file.name()
            );

            // Upload the edited file
            spawn(async move {
                let payload = MediaUploadPayload {
                    file: edited_file,
                    reference_type: Some(MediaReference::Category),
                    width: None,
                    height: None,
                };

                match media_state.upload(payload).await {
                    Ok(blob_url) => {
                        gloo_console::log!("[CategoryForm] Edited upload successful:", &blob_url);
                        let mut form_mut = form.write();
                        if field_name == "logo" {
                            form_mut.data.logo_blob_url = Some(blob_url);
                            // Clear existing media info to use upload tracking
                            form_mut.data.logo_filename = None;
                            form_mut.data.logo_size = None;
                        } else if field_name == "cover" {
                            form_mut.data.cover_blob_url = Some(blob_url);
                            // Clear existing media info to use upload tracking
                            form_mut.data.cover_filename = None;
                            form_mut.data.cover_size = None;
                        }
                    }
                    Err(e) => {
                        gloo_console::error!("[CategoryForm] Edited upload failed:", e);
                    }
                }
            });
        }
    };

    // Handle re-edit of already uploaded image
    let handle_edit_uploaded = move |field: String| {
        move |blob_url: String| {
            gloo_console::log!(
                "[CategoryForm] Re-editing uploaded image for",
                &field,
                ":",
                &blob_url
            );
            pending_field.set(Some(field.clone()));
            // Open editor directly with the blob URL
            spawn(async move {
                let _ = editor_state.open_editor(None, blob_url).await;
            });
        }
    };

    rsx! {
        div {
            if let Some(t) = props.title.clone() { h1 { class: "sr-only", {t} } }

            div { class: "grid grid-cols-1 gap-10 lg:grid-cols-3",
                // Main column
                div { class: "lg:col-span-2 space-y-8",
                    // Details card
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 py-6",
                            h2 { class: "text-lg font-semibold", "Category details" }
                            p { class: "text-sm text-muted-foreground", "Basic information and metadata for your category." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            // Name
                            AppInput { name: "name", form, label: "Name", placeholder: "e.g. Tutorials" }

                            div { class: "h-px bg-border/60" }

                            // Slug with generate
                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    label { class: "block text-sm font-medium text-foreground", "Slug" }
                                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            let name_value = form.peek().get_field("name").unwrap().value.clone();
                                            if !name_value.is_empty() {
                                                let sanitized = CategoryForm::sanitize_slug(&name_value);
                                                form.write().update_field("slug", sanitized);
                                            }
                                        },
                                        "Generate from name"
                                    }
                                }
                                AppInput { form, name: "slug", r#type: "text", placeholder: "tutorials" }
                                div { class: "flex items-center gap-2",
                                    span { class: "text-xs text-muted-foreground", "URL preview:" }
                                    code { class: "rounded border border-border/60 bg-transparent px-1.5 py-0.5 text-xs text-muted-foreground",
                                        {
                                            let slug = form.read().data.slug.clone();
                                            let safe = if slug.trim().is_empty() { "your-slug".to_string() } else { slug };
                                            format!("/categories/{}", safe)
                                        }
                                    }
                                }
                            }

                            div { class: "h-px bg-border/60" }

                            // Description
                            div { class: "space-y-3",
                                label { class: "block text-sm font-medium text-foreground", "Description" }
                                textarea {
                                    class: "w-full h-32 resize-none rounded-md border border-border/70 bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-muted-foreground transition-colors duration-200 focus:border-ring focus:ring-2 focus:ring-ring/40",
                                    placeholder: "Briefly describe what posts belong in this category.",
                                    value: form.read().data.description.clone(),
                                    oninput: move |event| { form.write().update_field("description", event.value()); }
                                }
                            }
                        }
                    }

                    // Branding card
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 py-6",
                            h2 { class: "text-lg font-semibold", "Images" }
                            p { class: "text-sm text-muted-foreground", "Logo and cover images for the category." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            // Logo image upload
                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    label { class: "block text-sm font-medium text-foreground", "Logo image" }
                                    p { class: "text-xs text-muted-foreground", "Square logo works best" }
                                }

                                {
                                    let form_data = form.read().data.clone();
                                    let has_logo_blob = form_data.logo_blob_url.is_some();
                                    let logo_blob_url = form_data.logo_blob_url.clone();

                                    rsx! {
                                        if has_logo_blob {
                                            // Show uploaded image with status
                                            {
                                                let blob = logo_blob_url.as_ref().unwrap();
                                                // Check if we have existing media info first, otherwise check upload tracking
                                                let (filename, file_size) = if let (Some(fname), Some(fsize)) = (&form_data.logo_filename, form_data.logo_size) {
                                                    // Use existing media info
                                                    (fname.clone(), fsize)
                                                } else if let Some(info) = media_state.get_file_info(blob) {
                                                    // Use upload tracking info for new uploads
                                                    (info.filename, info.size)
                                                } else {
                                                    // Fallback
                                                    ("Logo".to_string(), 0)
                                                };

                                                rsx! {
                                                    MediaUploadItem {
                                                        blob_url: blob.clone(),
                                                        filename,
                                                        file_size,
                                                        is_existing: form_data.logo_filename.is_some(),
                                                        on_remove: move |_url: String| {
                                                            let mut form_mut = form.write();
                                                            form_mut.data.logo_blob_url = None;
                                                            form_mut.data.logo_media_id = None;
                                                            form_mut.data.logo_filename = None;
                                                            form_mut.data.logo_size = None;
                                                        },
                                                        on_edit: Some(EventHandler::new(handle_edit_uploaded("logo".to_string()))),
                                                    }
                                                }
                                            }
                                        } else {
                                            MediaUploadZone {
                                                on_upload: move |_blob_urls: Vec<String>| {
                                                    // Not used - we use on_file_selected instead
                                                },
                                                on_file_selected: Some(EventHandler::new(handle_file_selected("logo".to_string()))),
                                                reference_type: Some(MediaReference::Category),
                                                max_files: 1,
                                                allowed_types: vec!["image/".to_string()],
                                                title: "Upload logo".to_string(),
                                                description: "Click to select an image file".to_string(),
                                                multiple: false,
                                            }
                                        }
                                    }
                                }
                            }

                            // Cover image upload
                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    label { class: "block text-sm font-medium text-foreground", "Cover image" }
                                    p { class: "text-xs text-muted-foreground", "Recommended 1200×600" }
                                }

                                {
                                    let form_data = form.read().data.clone();
                                    let has_cover_blob = form_data.cover_blob_url.is_some();
                                    let cover_blob_url = form_data.cover_blob_url.clone();

                                    rsx! {
                                        if has_cover_blob {
                                            // Show uploaded image with status
                                            {
                                                let blob = cover_blob_url.as_ref().unwrap();
                                                // Check if we have existing media info first, otherwise check upload tracking
                                                let (filename, file_size) = if let (Some(fname), Some(fsize)) = (&form_data.cover_filename, form_data.cover_size) {
                                                    // Use existing media info
                                                    (fname.clone(), fsize)
                                                } else if let Some(info) = media_state.get_file_info(blob) {
                                                    // Use upload tracking info for new uploads
                                                    (info.filename, info.size)
                                                } else {
                                                    // Fallback
                                                    ("Cover".to_string(), 0)
                                                };

                                                rsx! {
                                                    MediaUploadItem {
                                                        blob_url: blob.clone(),
                                                        filename,
                                                        file_size,
                                                        is_existing: form_data.cover_filename.is_some(),
                                                        on_remove: move |_url: String| {
                                                            let mut form_mut = form.write();
                                                            form_mut.data.cover_blob_url = None;
                                                            form_mut.data.cover_media_id = None;
                                                            form_mut.data.cover_filename = None;
                                                            form_mut.data.cover_size = None;
                                                        },
                                                        on_edit: Some(EventHandler::new(handle_edit_uploaded("cover".to_string()))),
                                                    }
                                                }
                                            }
                                        } else {
                                            MediaUploadZone {
                                                on_upload: move |_blob_urls: Vec<String>| {
                                                    // Not used - we use on_file_selected instead
                                                },
                                                on_file_selected: Some(EventHandler::new(handle_file_selected("cover".to_string()))),
                                                reference_type: Some(MediaReference::Category),
                                                max_files: 1,
                                                allowed_types: vec!["image/".to_string()],
                                                title: "Upload cover".to_string(),
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

                // Sidebar column
                div { class: "space-y-8 lg:sticky lg:top-28 h-fit",
                    // Visibility card
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Visibility" }
                            p { class: "text-sm text-muted-foreground", "Control whether this category is available publicly." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            div { class: "flex items-center justify-between",
                                div { class: "space-y-0.5",
                                    label { class: "block text-sm font-medium text-foreground", "Active" }
                                    p { class: "text-xs text-muted-foreground",
                                        if form.read().data.active { "This category will be visible across your site." } else { "This category will be hidden and unavailable for selection." }
                                    }
                                }
                                Checkbox { class: None, checked: form.read().data.active, onchange: move |checked: bool| { form.write().update_field("active", checked.to_string()); } }
                            }
                        }
                    }

                    // Branding
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Branding" }
                            p { class: "text-sm text-muted-foreground", "Pick a color and optionally override the text color." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            // Color picker + preview
                            div { class: "space-y-3",
                                label { class: "block text-sm font-medium text-foreground", "Category color" }
                                ColorPicker { value: form.read().data.color.clone(), onchange: move |val| { form.write().update_field("color", val); } }
                                {
                                    let data = form.read().data.clone();
                                    let color = data.color.clone();
                                    rsx! {
                                        div { class: "flex items-center gap-3",
                                            code { class: "text-xs rounded border border-border/60 bg-transparent px-1.5 py-0.5 text-muted-foreground", {color} }
                                        }
                                        p { class: "text-xs text-muted-foreground", if data.custom_text_color { "Using custom text color." } else { "Text color auto-adjusts for readability." } }
                                    }
                                }
                            }

                            // Optional custom text color toggle
                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    div { class: "space-y-0.5",
                                        label { class: "block text-sm font-medium text-foreground", "Text color" }
                                        p { class: "text-xs text-muted-foreground", "Enable to choose a custom text color. Otherwise it auto-adjusts for readability." }
                                    }
                                    Checkbox { class: None, checked: form.read().data.custom_text_color, onchange: move |checked: bool| { form.write().update_field("custom_text_color", checked.to_string()); } }
                                }
                                if form.read().data.custom_text_color {
                                    ColorPicker { value: form.read().data.text_color.clone(), onchange: move |val| { form.write().update_field("text_color", val); } }
                                }
                            }
                        }
                    }

                    // Parent category selector
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Parent Category" }
                            p { class: "text-sm text-muted-foreground", "Optional. Select a parent to create a hierarchy." }
                        }
                        div { class: "px-6 py-6",
                            {
                                let cats = cats_state;
                                let list = cats.list.read();
                                let is_loading = list.is_loading();
                                let is_failed = list.is_failed();
                                let message = list.error_message();
                                let sub_text = message.clone().unwrap_or_else(|| "There was a problem loading categories. Please try again.".to_string());
                                let items: Vec<ComboboxItem> = list
                                    .data
                                    .clone()
                                    .map(|paginated| paginated.data)
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|c| ComboboxItem { value: c.id.to_string(), label: c.name })
                                    .collect();
                                let current_val = {
                                    let pid = form.read().data.parent_id.clone();
                                    if pid.trim().is_empty() { None } else { Some(pid) }
                                };
                                rsx! {
                                    if is_loading && items.is_empty() {
                                        Skeleton { class: Some("h-10 w-full".to_string()) }
                                    } else if is_failed {
                                        div { class: "rounded-md border border-red-200 bg-red-50 p-3 text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-300",
                                            div { class: "flex flex-col items-center text-center gap-3",
                                                div { class: "space-y-1",
                                                    p { class: "text-sm font-semibold", "Failed to fetch categories list" }
                                                    p { class: "text-xs opacity-80", { sub_text } }
                                                }
                                                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                                                    onclick: move |_| {
                                                        let cats = cats;
                                                        spawn(async move { cats.list().await; });
                                                    },
                                                    "Retry"
                                                }
                                            }
                                        }
                                    } else {
                                        Combobox {
                                            items,
                                            placeholder: "Select parent...".to_string(),
                                            value: current_val,
                                            width: "w-full".to_string(),
                                            onvaluechange: Some(EventHandler::new(move |val: Option<String>| {
                                                let v = val.unwrap_or_default();
                                                form.write().update_field("parent_id", v);
                                            })),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Actions
                    {
                        let form_data = form.read().data.clone();
                        let is_uploading = form_data.is_uploading();

                        rsx! {
                            if is_uploading {
                                // Show upload in progress message
                                div { class: "flex items-center justify-center gap-2 py-3 px-4 rounded-md bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-800",
                                    Icon { icon: LdLoader, class: "h-4 w-4 text-blue-600 dark:text-blue-400 animate-spin" }
                                    span { class: "text-sm text-blue-600 dark:text-blue-400",
                                        "Uploading images..."
                                    }
                                }
                            }

                            div { class: "flex gap-3 pt-4",
                                Button { class: "flex-1 w-auto", variant: ButtonVariant::Outline,
                                    onclick: move |_| {
                                        if form.peek().is_dirty() {
                                            reset_dialog_open.set(true);
                                        } else {
                                            nav.push(Route::CategoriesListScreen {});
                                        }
                                    },
                                    {if is_form_dirty { "Reset" } else { "Cancel" }}
                                }
                                Button {
                                    class: "flex-1 w-auto",
                                    disabled: is_uploading,
                                    onclick: move |_| {
                                        if !is_uploading {
                                            let submit = props.on_submit.clone();
                                            form.write().on_submit(move |val| { submit.call(val); });
                                        }
                                    },
                                    {props.submit_label.clone().unwrap_or_else(|| "Save Category".to_string())}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Edit confirmation dialog
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

        if reset_dialog_open() {
            AppPortal {
                class: "bg-black/20 backdrop-blur-sm flex items-center justify-center px-4",
                div { class: "w-full max-w-md rounded-lg border border-border/60 bg-background p-6 shadow-lg",
                    div { class: "space-y-2",
                        h2 { class: "text-lg font-semibold", "Reset form?" }
                        p { class: "text-sm text-muted-foreground", "All changes will be cleared and the form will return to its default state." }
                    }
                    div { class: "mt-6 flex justify-end gap-2",
                        Button { variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                reset_dialog_open.set(false);
                            },
                            "Cancel"
                        }
                        Button { variant: ButtonVariant::Destructive,
                            onclick: move |_| {
                                form.set(OxForm::new(reset_template.clone()));
                                reset_dialog_open.set(false);
                            },
                            "Reset form"
                        }
                    }
                }
            }
        }
    }
}
