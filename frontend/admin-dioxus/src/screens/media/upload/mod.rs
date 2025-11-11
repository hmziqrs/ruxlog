use dioxus::prelude::*;

use crate::components::{MediaUploadList, MediaUploadZone, PageHeader};
use crate::router::Route;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn MediaUploadScreen() -> Element {
    let nav = use_navigator();
    let uploaded_blob_urls = use_signal(|| Vec::<String>::new());

    let handle_upload = {
        let mut uploaded_blob_urls = uploaded_blob_urls;
        move |blob_urls: Vec<String>| {
            let mut current = uploaded_blob_urls.peek().clone();
            current.extend(blob_urls);
            uploaded_blob_urls.set(current);
        }
    };

    let handle_remove = {
        let mut uploaded_blob_urls = uploaded_blob_urls;
        move |blob_url: String| {
            let mut current = uploaded_blob_urls.peek().clone();
            current.retain(|url| url != &blob_url);
            uploaded_blob_urls.set(current);
        }
    };

    let has_uploads = !uploaded_blob_urls.read().is_empty();

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            PageHeader {
                title: "Upload Media".to_string(),
                description: "Upload images, videos, and other media files".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.push(Route::MediaListScreen {}); },
                        "Back to Media"
                    }
                }),
                class: None,
                embedded: false,
            }

            div { class: "container mx-auto px-4 py-10 md:py-12 space-y-8",
                div { class: "max-w-3xl mx-auto",
                    MediaUploadZone {
                        on_upload: handle_upload,
                        reference_type: None,
                        max_files: 0, // Unlimited
                        allowed_types: vec![
                            "image/".to_string(),
                            "video/".to_string(),
                        ],
                        title: "Upload Media Files".to_string(),
                        description: "Drag and drop files here, or click to select. Supports images and videos.".to_string(),
                        multiple: true,
                    }
                }
                if has_uploads {
                    div { class: "max-w-5xl mx-auto space-y-4",
                        div { class: "flex items-center justify-between",
                            h2 { class: "text-lg font-semibold", "Upload Progress" }
                            div { class: "flex items-center gap-2",
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| {
                                        nav.push(Route::MediaListScreen {});
                                    },
                                    "View Gallery"
                                }
                            }
                        }
                        MediaUploadList {
                            blob_urls: uploaded_blob_urls(),
                            on_remove: handle_remove,
                        }
                    }
                }
            }
        }
    }
}
