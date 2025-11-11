use crate::store::{use_media, UploadStatus};
use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant, Progress};
use crate::utils::file_helpers::format_file_size;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCheck, LdCircle, LdLoader, LdPencil, LdX},
    Icon,
};

#[derive(Props, Clone, PartialEq)]
pub struct MediaUploadItemProps {
    /// The blob URL identifying this upload
    pub blob_url: String,
    /// File name to display
    pub filename: String,
    /// File size in bytes
    pub file_size: i64,
    /// Optional callback when user clicks remove/cancel
    #[props(default)]
    pub on_remove: Option<EventHandler<String>>,
    /// Optional callback when user clicks edit
    #[props(default)]
    pub on_edit: Option<EventHandler<String>>,
    /// Whether this is an existing media (not a new upload)
    #[props(default)]
    pub is_existing: bool,
}

/// Component that displays a single file upload with progress and status
#[component]
pub fn MediaUploadItem(props: MediaUploadItemProps) -> Element {
    let media_state = use_media();
    let blob_url = props.blob_url.clone();

    // Get current upload status and progress
    let status = if props.is_existing {
        // For existing media, always show as success
        Some(UploadStatus::Success)
    } else {
        media_state.get_upload_status(&blob_url)
    };
    let progress = media_state.get_upload_progress(&blob_url);

    let is_uploading = matches!(status, Some(UploadStatus::Uploading));
    let is_success = matches!(status, Some(UploadStatus::Success));
    let is_error = matches!(status, Some(UploadStatus::Error(_)));

    // Determine status classes and icons
    let (status_class, status_icon, status_text) = match &status {
        Some(UploadStatus::Uploading) => (
            "border-blue-200 dark:border-blue-800",
            rsx! { Icon { icon: LdLoader, class: "h-4 w-4 text-blue-600 dark:text-blue-400 animate-spin" } },
            "Uploading...",
        ),
        Some(UploadStatus::Success) => (
            "border-green-200 dark:border-green-800",
            rsx! { Icon { icon: LdCheck, class: "h-4 w-4 text-green-600 dark:text-green-400" } },
            "Uploaded",
        ),
        Some(UploadStatus::Error(err)) => (
            "border-red-200 dark:border-red-800",
            rsx! { Icon { icon: LdCircle, class: "h-4 w-4 text-red-600 dark:text-red-400" } },
            err.as_str(),
        ),
        None => (
            "border-zinc-200 dark:border-zinc-800",
            rsx! { Icon { icon: LdLoader, class: "h-4 w-4 text-zinc-400 animate-spin" } },
            "Preparing...",
        ),
    };

    let handle_remove = {
        let blob_url = blob_url.clone();
        let on_remove = props.on_remove.clone();
        move |_| {
            if let Some(handler) = &on_remove {
                handler.call(blob_url.clone());
            }
        }
    };

    let handle_edit = {
        let blob_url = blob_url.clone();
        let on_edit = props.on_edit.clone();
        move |_| {
            if let Some(handler) = &on_edit {
                handler.call(blob_url.clone());
            }
        }
    };

    rsx! {
        div { class: "border rounded-lg p-3 space-y-2 {status_class} transition-colors",
            // Top row: thumbnail + info + actions
            div { class: "flex items-start gap-3",
                // Thumbnail preview
                div { class: "flex-shrink-0",
                    img {
                        src: "{blob_url}",
                        alt: "{props.filename}",
                        class: "w-16 h-16 object-cover rounded border border-zinc-200 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-800",
                    }
                }

                // File info
                div { class: "flex-1 min-w-0 space-y-1",
                    p { class: "text-sm font-medium text-foreground truncate",
                        "{props.filename}"
                    }
                    p { class: "text-xs text-muted-foreground",
                        "{format_file_size(props.file_size)}"
                    }
                    div { class: "flex items-center gap-2 text-xs",
                        {status_icon}
                        span { class: if is_error { "text-red-600 dark:text-red-400" } else { "text-muted-foreground" },
                            "{status_text}"
                        }
                    }
                }

                // Action buttons
                div { class: "flex items-center gap-1",
                    // Edit button (only show when upload is successful and on_edit is provided)
                    if is_success && props.on_edit.is_some() {
                        Button {
                            r#type: "button".to_string(),
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            onclick: handle_edit,
                            class: "h-8 w-8 flex-shrink-0".to_string(),
                            Icon { icon: LdPencil, class: "h-4 w-4" }
                            span { class: "sr-only", "Edit" }
                        }
                    }

                    // Remove/Cancel button
                    if props.on_remove.is_some() {
                        Button {
                            r#type: "button".to_string(),
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            onclick: handle_remove,
                            class: "h-8 w-8 flex-shrink-0".to_string(),
                            Icon { icon: LdX, class: "h-4 w-4" }
                            span { class: "sr-only", "Remove" }
                        }
                    }
                }
            }

            // Progress bar (only show during upload)
            if is_uploading {
                Progress {
                    value: progress as i32,
                    class: Some("h-1".to_string()),
                }
            }
        }
    }
}
