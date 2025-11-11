//! MediaPickerDialog - A modal dialog for browsing and selecting media files
//! or uploading new ones. Used by the RichTextEditor and other components.

use crate::components::MediaUploadZone;
use crate::store::{use_media, Media, MediaListQuery, MediaReference};
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Badge, Button, ButtonVariant, Checkbox};
use crate::utils::dates::format_short_date_dt;
use crate::utils::file_helpers::{format_file_size, is_image};
use dioxus::prelude::*;
use gloo_timers::future::sleep;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdChevronLeft, LdChevronRight, LdUpload, LdX},
    Icon,
};
use std::time::Duration;

#[derive(Props, Clone, PartialEq)]
pub struct MediaPickerDialogProps {
    /// Whether the dialog is open
    pub open: bool,
    /// Callback when dialog is closed
    pub on_close: EventHandler<()>,
    /// Callback when media is selected
    pub on_select: EventHandler<Media>,
    /// Optional filter for media types (e.g., "image/*")
    #[props(default)]
    pub filter_type: Option<String>,
    /// Optional reference type for new uploads
    #[props(default)]
    pub reference_type: Option<MediaReference>,
}

#[component]
pub fn MediaPickerDialog(props: MediaPickerDialogProps) -> Element {
    let media_state = use_media();
    let mut current_tab = use_signal(|| "browse".to_string());
    let mut filters = use_signal(|| MediaListQuery::new());
    let mut selected_ids = use_signal(|| Vec::<i32>::new());
    let mut upload_blob_urls = use_signal(|| Vec::<String>::new());

    // Load media list when dialog opens
    use_effect({
        let media_state = media_state;
        move || {
            if props.open {
                let q = filters();
                spawn(async move {
                    media_state.list_with_query(q).await;
                });
            }
        }
    });

    let list = media_state.list.read();
    let list_loading = list.is_loading();

    let (media_items, current_page, total_pages) = if let Some(p) = &list.data {
        let total_pages = if p.per_page > 0 {
            ((p.total as f64) / (p.per_page as f64)).ceil() as u64
        } else {
            1
        };
        (p.data.clone(), p.page, total_pages)
    } else {
        (Vec::<Media>::new(), 1, 1)
    };

    // Clone filter_type early to use in multiple places
    let filter_type_clone = props.filter_type.clone();

    // Filter by type if specified
    let filtered_items = use_memo(move || {
        let items = media_items.clone();
        if let Some(ref ft) = filter_type_clone {
            if ft.starts_with("image/") {
                items
                    .into_iter()
                    .filter(|m| is_image(&m.mime_type))
                    .collect()
            } else {
                items
            }
        } else {
            items
        }
    });

    let has_data = !filtered_items.read().is_empty();
    let items_vec = filtered_items.read().clone();

    // Handle page navigation
    let handle_prev_page = move |_| {
        let mut q = filters();
        if current_page > 1 {
            q.page = current_page - 1;
            filters.set(q.clone());
            spawn(async move {
                let media_state = use_media();
                media_state.list_with_query(q).await;
            });
        }
    };

    let handle_next_page = move |_| {
        let mut q = filters();
        if current_page < total_pages {
            q.page = current_page + 1;
            filters.set(q.clone());
            spawn(async move {
                let media_state = use_media();
                media_state.list_with_query(q).await;
            });
        }
    };

    // Handle media selection
    let handle_select = move |media: Media| {
        props.on_select.call(media);
        props.on_close.call(());
    };

    // Handle upload completion
    let handle_upload = move |blob_urls: Vec<String>| {
        upload_blob_urls.set(blob_urls.clone());
        // Wait for upload to complete, then refresh list and switch to browse tab
        spawn(async move {
            sleep(Duration::from_millis(2000)).await;
            let media_state = use_media();
            let q = filters();
            media_state.list_with_query(q).await;
            current_tab.set("browse".to_string());
            upload_blob_urls.set(Vec::new());
        });
    };

    if !props.open {
        return rsx! {};
    }

    rsx! {
        AppPortal {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
                onclick: move |_| props.on_close.call(()),
                div {
                    class: "bg-base-100 rounded-lg shadow-xl w-full max-w-4xl max-h-[80vh] flex flex-col",
                    onclick: move |e| e.stop_propagation(),

                    // Header
                    div { class: "flex items-center justify-between p-4 border-b border-base-300",
                        h2 { class: "text-xl font-semibold text-primary", "Select Media" }
                        button {
                            class: "btn btn-ghost btn-sm btn-circle",
                            onclick: move |_| props.on_close.call(()),
                            Icon { icon: LdX, width: 20, height: 20 }
                        }
                    }

                    // Tabs
                    div { class: "flex border-b border-base-300",
                        button {
                            class: if current_tab() == "browse" {
                                "px-4 py-2 text-sm font-medium border-b-2 border-primary text-primary"
                            } else {
                                "px-4 py-2 text-sm font-medium text-base-content/60 hover:text-primary"
                            },
                            onclick: move |_| current_tab.set("browse".to_string()),
                            "Browse Media"
                        }
                        button {
                            class: if current_tab() == "upload" {
                                "px-4 py-2 text-sm font-medium border-b-2 border-primary text-primary"
                            } else {
                                "px-4 py-2 text-sm font-medium text-base-content/60 hover:text-primary"
                            },
                            onclick: move |_| current_tab.set("upload".to_string()),
                            "Upload New"
                        }
                    }

                    // Content
                    div { class: "flex-1 overflow-y-auto p-4",
                        if current_tab() == "browse" {
                            // Browse tab
                            if list_loading {
                                div { class: "flex items-center justify-center py-12",
                                    span { class: "loading loading-spinner loading-lg" }
                                }
                            } else if !has_data {
                                div { class: "flex flex-col items-center justify-center py-12 text-center",
                                    Icon { icon: LdUpload, width: 48, height: 48, class: "text-base-content/40 mb-4" }
                                    p { class: "text-base-content/60 mb-2", "No media files found" }
                                    Button {
                                        variant: ButtonVariant::Default,
                                        onclick: move |_| current_tab.set("upload".to_string()),
                                        "Upload Files"
                                    }
                                }
                            } else {
                                div { class: "overflow-x-auto",
                                    table { class: "table w-full",
                                        thead {
                                            tr {
                                                th { class: "w-12" }
                                                th { "Preview" }
                                                th { "File Name" }
                                                th { "Type" }
                                                th { "Size" }
                                                th { "Uploaded" }
                                            }
                                        }
                                        tbody {
                                            for media in items_vec.iter() {
                                                {
                                                    let media_clone = media.clone();
                                                    let media_id = media.id;
                                                    let media_mime = media.mime_type.clone();
                                                    let media_file_url = media.file_url.clone();
                                                    let media_obj_key = media.object_key.clone();
                                                    let media_size = media.size;
                                                    let media_created = media.created_at.clone();
                                                    let is_selected = selected_ids().contains(&media_id);
                                                    rsx! {
                                                        tr {
                                                            key: "{media_id}",
                                                            class: if is_selected { "bg-primary/10 hover:bg-primary/20 cursor-pointer" } else { "hover:bg-base-200 cursor-pointer" },
                                                            onclick: move |_| handle_select(media_clone.clone()),
                                                            td { class: "w-12 py-2 px-3",
                                                                Checkbox {
                                                                    checked: is_selected,
                                                                    onchange: move |_| {
                                                                        let mut ids = selected_ids();
                                                                        if ids.contains(&media_id) {
                                                                            ids.retain(|id| *id != media_id);
                                                                        } else {
                                                                            ids.push(media_id);
                                                                        }
                                                                        selected_ids.set(ids);
                                                                    }
                                                                }
                                                            }
                                                            td { class: "w-16 py-2 px-3",
                                                                if is_image(&media_mime) {
                                                                    img {
                                                                        src: "{media_file_url}",
                                                                        alt: "{media_obj_key}",
                                                                        class: "w-12 h-12 object-cover rounded"
                                                                    }
                                                                } else {
                                                                    div { class: "w-12 h-12 bg-base-300 rounded flex items-center justify-center text-xs text-base-content/60",
                                                                        "{media_mime.split('/').last().unwrap_or(\"file\").to_uppercase()}"
                                                                    }
                                                                }
                                                            }
                                                            td { class: "py-2 px-3 text-sm",
                                                                div { class: "truncate max-w-xs", "{media_obj_key}" }
                                                            }
                                                            td { class: "py-2 px-3 text-sm",
                                                                Badge { class: "badge-ghost", "{media_mime}" }
                                                            }
                                                            td { class: "py-2 px-3 text-sm",
                                                                "{format_file_size(media_size)}"
                                                            }
                                                            td { class: "py-2 px-3 text-sm text-base-content/60",
                                                                "{format_short_date_dt(&media_created)}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Pagination
                                if total_pages > 1 {
                                    div { class: "flex items-center justify-center gap-4 mt-4",
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            disabled: current_page <= 1,
                                            onclick: handle_prev_page,
                                            Icon { icon: LdChevronLeft, width: 16, height: 16 }
                                            "Previous"
                                        }
                                        span { class: "text-sm text-base-content/60",
                                            "Page {current_page} of {total_pages}"
                                        }
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            disabled: current_page >= total_pages,
                                            onclick: handle_next_page,
                                            "Next"
                                            Icon { icon: LdChevronRight, width: 16, height: 16 }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Upload tab
                            div { class: "space-y-4",
                                MediaUploadZone {
                                    on_upload: handle_upload,
                                    reference_type: props.reference_type.clone().unwrap_or(MediaReference::Post),
                                    max_files: 10,
                                    allowed_types: props.filter_type.as_ref().map(|ft| vec![ft.clone()]).unwrap_or_default(),
                                }

                                if !upload_blob_urls().is_empty() {
                                    div { class: "alert alert-info",
                                        "Uploading {upload_blob_urls().len()} file(s)... You'll be redirected to browse tab when complete."
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
