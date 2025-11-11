use dioxus::prelude::*;
use gloo_console;

use web_sys::{Blob, Url};

use super::form::{use_user_form, UserForm};
use crate::components::{
    AppInput, ConfirmDialog, ImageEditorModal, MediaPreviewItem, MediaUploadItem, MediaUploadZone,
    PasswordInput,
};
use crate::hooks::OxForm;
use crate::router::Route;
use crate::store::{use_image_editor, use_media, MediaReference, MediaUploadPayload};
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Button, ButtonVariant, Checkbox};

#[derive(Props, PartialEq, Clone)]
pub struct UserFormContainerProps {
    #[props(default)]
    pub initial: Option<UserForm>,
    pub on_submit: EventHandler<UserForm>,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub submit_label: Option<String>,
}

#[component]
pub fn UserFormContainer(props: UserFormContainerProps) -> Element {
    let nav = use_navigator();
    let initial_user_form = props.initial.clone().unwrap_or_else(UserForm::new);
    let reset_template = initial_user_form.clone();
    let user_form_hook = use_user_form(initial_user_form);
    let mut form = user_form_hook.form;
    let mut reset_dialog_open = use_signal(|| false);
    let is_form_dirty = form.read().is_dirty();
    let is_update = form.read().data.is_update;

    // Media upload state
    let media_state = use_media();
    let editor_state = use_image_editor();
    let mut pending_file = use_signal(|| None::<web_sys::File>);
    let mut confirm_dialog_open = use_signal(|| false);
    let _is_uploading = media_state.upload.read().is_loading();

    // Handle file selected from MediaUploadZone
    let handle_file_selected = move |files: Vec<web_sys::File>| {
        if let Some(file) = files.first() {
            gloo_console::log!("[UserForm] File selected:", file.name());
            pending_file.set(Some(file.clone()));
            confirm_dialog_open.set(true);
        }
    };

    // Handle edit confirmation - open image editor
    let handle_edit_confirm = move |_| {
        let file = pending_file();
        if let Some(f) = file {
            gloo_console::log!("[UserForm] Opening editor for file:", f.name());
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
        if let Some(f) = file {
            gloo_console::log!("[UserForm] Skipping edit, uploading directly:", f.name());

            spawn(async move {
                let payload = MediaUploadPayload {
                    file: f.clone(),
                    reference_type: Some(MediaReference::User),
                    width: None,
                    height: None,
                };

                match media_state.upload(payload).await {
                    Ok(blob_url) => {
                        gloo_console::log!("[UserForm] Upload successful:", &blob_url);
                        form.write().data.avatar_blob_url = Some(blob_url);
                    }
                    Err(e) => {
                        gloo_console::error!("[UserForm] Upload failed:", e);
                    }
                }
            });
        }
    };

    // Handle image editor save - upload edited file
    let handle_editor_save = move |edited_file: web_sys::File| {
        gloo_console::log!(
            "[UserForm] Editor saved, uploading edited file:",
            edited_file.name()
        );

        spawn(async move {
            let payload = MediaUploadPayload {
                file: edited_file,
                reference_type: Some(MediaReference::User),
                width: None,
                height: None,
            };

            match media_state.upload(payload).await {
                Ok(blob_url) => {
                    gloo_console::log!("[UserForm] Edited upload successful:", &blob_url);
                    form.write().data.avatar_blob_url = Some(blob_url);
                }
                Err(e) => {
                    gloo_console::error!("[UserForm] Edited upload failed:", e);
                }
            }
        });
    };

    // Handle edit uploaded avatar
    let handle_edit_uploaded = move |blob_url: String| {
        gloo_console::log!("[UserForm] Editing uploaded avatar");
        spawn(async move {
            let _ = editor_state.open_editor(None, blob_url).await;
        });
    };

    // Track upload status and resolve media ID
    use_effect(move || {
        let form_data = form.read().data.clone();

        // Check avatar blob URL
        if let Some(avatar_blob) = &form_data.avatar_blob_url {
            if form_data.avatar_id.is_none() {
                // Check if upload completed
                if let Some(media) = media_state.get_uploaded_media(avatar_blob) {
                    gloo_console::log!(
                        "[UserForm] Avatar upload complete, media ID:",
                        media.id.to_string()
                    );
                    let mut form_mut = form.write();
                    form_mut.data.avatar_id = Some(media.id);
                }
            }
        }
    });

    rsx! {
        div {
            if let Some(t) = props.title.clone() { h1 { class: "sr-only", {t} } }

            div { class: "grid grid-cols-1 gap-10 lg:grid-cols-3",
                div { class: "lg:col-span-2 space-y-8",
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 py-6",
                            h2 { class: "text-lg font-semibold", "User details" }
                            p { class: "text-sm text-muted-foreground", "Basic information and authentication credentials." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            AppInput {
                                name: "name",
                                form,
                                label: "Name",
                                placeholder: "e.g. John Doe"
                            }

                            div { class: "h-px bg-border/60" }

                            AppInput {
                                name: "email",
                                form,
                                label: "Email",
                                r#type: "email",
                                placeholder: "user@example.com"
                            }

                            div { class: "h-px bg-border/60" }

                            PasswordInput {
                                name: "password",
                                form,
                                label: if is_update { "Password (optional)" } else { "Password" },
                                placeholder: if is_update { "Leave blank to keep current password" } else { "Enter password (min 8 characters)" },
                                onchange: move |_| {
                                    // Re-validate confirm password when password changes
                                    if form.peek().submit_count > 0 {
                                        let form_data = form.peek().data.clone();
                                        let password = form_data.password.as_deref().unwrap_or("");
                                        let confirm = form_data.confirm_password.as_deref().unwrap_or("");

                                        if !form_data.is_update || !password.is_empty() {
                                            if password != confirm && !confirm.is_empty() {
                                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                                    field.set_error(Some("Passwords do not match".to_string()));
                                                }
                                            } else if password == confirm {
                                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                                    field.set_error(None);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if is_update {
                                p { class: "text-xs text-muted-foreground", "Leave blank to keep the current password unchanged." }
                            } else {
                                p { class: "text-xs text-muted-foreground", "Password must be at least 8 characters long." }
                            }

                            div { class: "h-px bg-border/60" }

                            PasswordInput {
                                name: "confirm_password",
                                form,
                                label: if is_update { "Confirm Password (optional)" } else { "Confirm Password" },
                                placeholder: if is_update { "Re-enter new password" } else { "Re-enter password" },
                                onchange: move |_| {
                                    // Validate password matching on change
                                    if form.peek().submit_count > 0 {
                                        let form_data = form.peek().data.clone();
                                        let password = form_data.password.as_deref().unwrap_or("");
                                        let confirm = form_data.confirm_password.as_deref().unwrap_or("");

                                        if !form_data.is_update || !password.is_empty() {
                                            if password != confirm && !confirm.is_empty() {
                                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                                    field.set_error(Some("Passwords do not match".to_string()));
                                                }
                                            } else if password == confirm {
                                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                                    field.set_error(None);
                                                }
                                            }
                                        }
                                    }
                                },
                                onblur: move |_| {
                                    // Validate password matching on blur
                                    if form.peek().submit_count > 0 {
                                        let form_data = form.peek().data.clone();
                                        let password = form_data.password.as_deref().unwrap_or("");
                                        let confirm = form_data.confirm_password.as_deref().unwrap_or("");

                                        if !form_data.is_update || !password.is_empty() {
                                            if password != confirm {
                                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                                    field.set_error(Some("Passwords do not match".to_string()));
                                                }
                                            } else {
                                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                                    field.set_error(None);
                                                }
                                            }
                                        }
                                    }

                                }
                            }
                            if !is_update {
                                p { class: "text-xs text-muted-foreground", "Both passwords must match." }
                            }
                        }
                    }

                    div { class: "flex items-start gap-3 rounded-lg border border-border/60 bg-transparent p-5",
                        div { class: "mt-0.5 h-4 w-4 rounded-full border border-border/40" }
                        div { class: "space-y-1",
                            p { class: "text-sm font-medium text-foreground", "Security tip" }
                            p { class: "text-sm text-muted-foreground", "Ensure users verify their email and use strong passwords. Super admins have full system access." }
                        }
                    }
                }

                div { class: "space-y-8 lg:sticky lg:top-28 h-fit",
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Avatar" }
                            p { class: "text-sm text-muted-foreground", "Upload a profile picture for this user." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            div { class: "space-y-3",
                                label { class: "block text-sm font-medium text-foreground", "Profile Picture" }
                                {
                                    let form_data = form.read().data.clone();

                                    if let Some(blob) = form_data.avatar_blob_url {
                                        // Show newly uploaded avatar (blob URL)
                                        let file_info = media_state.get_file_info(&blob);
                                        let (filename, file_size) = if let Some(info) = file_info {
                                            (info.filename, info.size)
                                        } else {
                                            ("Avatar".to_string(), 0)
                                        };
                                        rsx! {
                                            MediaUploadItem {
                                                blob_url: blob.clone(),
                                                filename,
                                                file_size,
                                                on_remove: move |_url: String| {
                                                    let mut form_mut = form.write();
                                                    form_mut.data.avatar_blob_url = None;
                                                    form_mut.data.avatar_id = None;
                                                },
                                                on_edit: Some(EventHandler::new(handle_edit_uploaded)),
                                            }
                                        }
                                    } else if let Some(existing_avatar) = form_data.existing_avatar {
                                        // Show existing avatar from server
                                        rsx! {
                                            MediaPreviewItem {
                                                media: existing_avatar.clone(),
                                                alt: form_data.name.clone(),
                                                on_remove: Some(EventHandler::new(move |_id: i32| {
                                                    let mut form_mut = form.write();
                                                    form_mut.data.existing_avatar = None;
                                                    form_mut.data.avatar_id = None;
                                                })),
                                                on_edit: Some(EventHandler::new(handle_edit_uploaded)),
                                            }
                                        }
                                    } else {
                                        // Show upload zone
                                        rsx! {
                                            MediaUploadZone {
                                                on_upload: move |_blob_urls: Vec<String>| {
                                                    // Not used - we use on_file_selected instead
                                                },
                                                on_file_selected: Some(EventHandler::new(handle_file_selected)),
                                                reference_type: Some(MediaReference::User),
                                                max_files: 1,
                                                allowed_types: vec!["image/".to_string()],
                                                title: "Upload avatar".to_string(),
                                                description: "Click to select an image file".to_string(),
                                                multiple: false,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Role & Permissions" }
                            p { class: "text-sm text-muted-foreground", "Assign role and set verification status." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            div { class: "space-y-3",
                                label { class: "block text-sm font-medium text-foreground", "Role" }
                                select {
                                    class: "w-full rounded-md border border-border/70 bg-transparent px-3 py-2 text-sm text-foreground transition-colors duration-200 focus:border-ring focus:ring-2 focus:ring-ring/40",
                                    value: form.read().data.role.to_string(),
                                    onchange: move |e| {
                                        form.write().update_field("role", e.value());
                                    },
                                    option { value: "super-admin", "Super Admin" }
                                    option { value: "admin", "Admin" }
                                    option { value: "moderator", "Moderator" }
                                    option { value: "author", "Author" }
                                    option { value: "user", "User" }
                                }
                                p { class: "text-xs text-muted-foreground", "Determines what actions this user can perform." }
                            }

                            div { class: "h-px bg-border/60" }

                            div { class: "flex items-center justify-between",
                                div { class: "space-y-0.5",
                                    label { class: "block text-sm font-medium text-foreground", "Email Verified" }
                                    p { class: "text-xs text-muted-foreground",
                                        if form.read().data.is_verified {
                                            "User has verified their email address."
                                        } else {
                                            "User has not verified their email address."
                                        }
                                    }
                                }
                                Checkbox {
                                    class: None,
                                    checked: form.read().data.is_verified,
                                    onchange: move |checked: bool| {
                                        form.write().update_field("is_verified", checked.to_string());
                                    }
                                }
                            }
                        }
                    }

                    div { class: "flex gap-3 pt-4",
                        Button { class: "flex-1 w-auto", variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                if form.peek().is_dirty() {
                                    reset_dialog_open.set(true);
                                } else {
                                    nav.push(Route::UsersListScreen {});
                                }
                            },
                            {if is_form_dirty { "Reset" } else { "Cancel" }}
                        }
                        Button { class: "flex-1 w-auto",
                            onclick: move |_| {
                                let submit = props.on_submit.clone();

                                // Check if passwords match before submitting
                                let form_data = form.peek().data.clone();
                                let password = form_data.password.as_deref().unwrap_or("");
                                let confirm = form_data.confirm_password.as_deref().unwrap_or("");

                                // For new users or when changing password, validate matching
                                if !form_data.is_update || !password.is_empty() {
                                    if password != confirm {
                                        // Set error on confirm_password field and trigger validation
                                        form.write().submit_count += 1;
                                        if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                            field.set_error(Some("Passwords do not match".to_string()));
                                        }
                                        form.write().has_errors = true;
                                        return;
                                    }

                                    // For new users, password is required
                                    if !form_data.is_update && password.is_empty() {
                                        form.write().submit_count += 1;
                                        if let Some(field) = form.write().fields.get_mut("password") {
                                            field.set_error(Some("Password is required".to_string()));
                                        }
                                        form.write().has_errors = true;
                                        return;
                                    }
                                }

                                // Clear confirm_password error if validation passes
                                if let Some(field) = form.write().fields.get_mut("confirm_password") {
                                    field.set_error(None);
                                }

                                form.write().on_submit(move |val| { submit.call(val); });
                            },
                            {props.submit_label.clone().unwrap_or_else(|| "Save User".to_string())}
                        }
                    }
                }
            }
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
        ConfirmDialog {
            is_open: confirm_dialog_open,
            title: "Edit image before uploading?".to_string(),
            description: "You can crop, resize, rotate, or compress the image before uploading.".to_string(),
            confirm_label: "Edit Image".to_string(),
            cancel_label: "Skip & Upload".to_string(),
            on_confirm: handle_edit_confirm,
            on_cancel: handle_edit_skip,
        }

        ImageEditorModal {
            on_save: handle_editor_save,
        }
    }
}
