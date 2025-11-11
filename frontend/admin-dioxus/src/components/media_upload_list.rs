use crate::components::MediaUploadItem;
use crate::store::{use_media, UploadStatus};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MediaUploadListProps {
    /// List of blob URLs to display
    pub blob_urls: Vec<String>,
    /// Called when user removes an item
    pub on_remove: EventHandler<String>,
}

/// Component that displays a list of uploads with progress tracking
#[component]
pub fn MediaUploadList(props: MediaUploadListProps) -> Element {
    let media_state = use_media();

    // Calculate upload statistics
    let (uploading_count, success_count, error_count) = {
        let mut uploading = 0;
        let mut success = 0;
        let mut error = 0;

        for blob_url in &props.blob_urls {
            match media_state.get_upload_status(blob_url) {
                Some(UploadStatus::Uploading) => uploading += 1,
                Some(UploadStatus::Success) => success += 1,
                Some(UploadStatus::Error(_)) => error += 1,
                None => uploading += 1, // Preparing/not started yet
            }
        }

        (uploading, success, error)
    };

    let total = props.blob_urls.len();

    if total == 0 {
        return rsx! {};
    }

    rsx! {
        div { class: "space-y-4",
            // Summary header
            div { class: "flex items-center justify-between text-sm",
                div { class: "space-x-4",
                    span { class: "text-muted-foreground",
                        "Total: "
                        span { class: "font-medium text-foreground", "{total}" }
                    }
                    if uploading_count > 0 {
                        span { class: "text-blue-600 dark:text-blue-400",
                            "Uploading: "
                            span { class: "font-medium", "{uploading_count}" }
                        }
                    }
                    if success_count > 0 {
                        span { class: "text-green-600 dark:text-green-400",
                            "Uploaded: "
                            span { class: "font-medium", "{success_count}" }
                        }
                    }
                    if error_count > 0 {
                        span { class: "text-red-600 dark:text-red-400",
                            "Failed: "
                            span { class: "font-medium", "{error_count}" }
                        }
                    }
                }
            }

            // Upload items grid
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                for blob_url in &props.blob_urls {
                    {
                        let blob_url_str = blob_url.clone();

                        // Get file info from stored metadata or uploaded media
                        let (filename, file_size) = if let Some(file_info) = media_state.get_file_info(&blob_url_str) {
                            (file_info.filename, file_info.size)
                        } else if let Some(media) = media_state.get_uploaded_media(&blob_url_str) {
                            // Fallback to media object if file info not available
                            let name = media.object_key.split('/').last().unwrap_or("Unknown").to_string();
                            (name, media.size)
                        } else {
                            ("Uploading...".to_string(), 0)
                        };

                        rsx! {
                            MediaUploadItem {
                                key: "{blob_url_str}",
                                blob_url: blob_url_str.clone(),
                                filename: filename,
                                file_size: file_size,
                                on_remove: move |url: String| {
                                    props.on_remove.call(url);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
