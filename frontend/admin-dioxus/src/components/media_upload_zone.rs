use crate::store::{use_media, MediaReference, MediaUploadPayload};
use crate::ui::shadcn::{Button, ButtonVariant};
use crate::utils::file_helpers::validate_file_type;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdUpload, Icon};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

static INPUT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Props, Clone, PartialEq)]
pub struct MediaUploadZoneProps {
    /// Called when files are selected and upload initiated
    /// Returns Vec of blob URLs
    pub on_upload: EventHandler<Vec<String>>,
    /// Optional: Called when files are selected, BEFORE upload
    /// If provided, automatic upload is skipped and Files are passed to this callback
    /// The callback should handle upload manually
    #[props(default)]
    pub on_file_selected: Option<EventHandler<Vec<web_sys::File>>>,
    /// Reference type for uploaded media
    #[props(default)]
    pub reference_type: Option<MediaReference>,
    /// Maximum number of files to accept at once (0 = unlimited)
    #[props(default = 0)]
    pub max_files: usize,
    /// Allowed mime type prefixes (e.g., ["image/", "video/"])
    /// Empty vec means all types allowed
    #[props(default)]
    pub allowed_types: Vec<String>,
    /// Title text for the upload zone
    #[props(default = "Upload Files".to_string())]
    pub title: String,
    /// Description text
    #[props(default = "Drag and drop files here, or click to select".to_string())]
    pub description: String,
    /// Whether to allow multiple files
    #[props(default = true)]
    pub multiple: bool,
}

/// Upload zone for media files (drag-and-drop temporarily disabled)
#[component]
pub fn MediaUploadZone(props: MediaUploadZoneProps) -> Element {
    let input_id = use_signal(|| {
        format!(
            "media-upload-zone-{}",
            INPUT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    });

    // Pre-clone props for event handlers
    let reference_type = props.reference_type.clone();
    let max_files = props.max_files;
    let allowed_types = props.allowed_types.clone();
    let on_upload_handler = props.on_upload.clone();

    // Open file picker
    let open_picker = {
        let id = input_id();
        move |_| {
            gloo_console::log!("[MediaUploadZone] Opening file picker");
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(el) = doc.get_element_by_id(&id) {
                        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                            let _ = input.click();
                        }
                    }
                }
            }
        }
    };

    // Handle file selection
    let handle_file_change = move |_evt: Event<FormData>| {
        gloo_console::log!("[MediaUploadZone] File change event triggered");

        let id = input_id();
        let reference_type_clone = reference_type.clone();
        let allowed_types_clone = allowed_types.clone();
        let on_upload_clone = on_upload_handler.clone();
        let on_file_selected_clone = props.on_file_selected.clone();

        spawn(async move {
            // Get the input element by ID
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(el) = doc.get_element_by_id(&id) {
                        if let Ok(target) = el.dyn_into::<HtmlInputElement>() {
                            if let Some(files) = target.files() {
                                let file_count = files.length();
                                gloo_console::log!(
                                    "[MediaUploadZone] Files selected: ",
                                    file_count.to_string()
                                );

                                if file_count > 0 {
                                    let allowed_refs: Vec<&str> =
                                        allowed_types_clone.iter().map(|s| s.as_str()).collect();
                                    gloo_console::log!(
                                        "[MediaUploadZone] Processing ",
                                        file_count.to_string(),
                                        " files with max: ",
                                        max_files.to_string()
                                    );

                                    // If on_file_selected is provided, extract Files and pass them
                                    if let Some(on_file_selected) = on_file_selected_clone {
                                        gloo_console::log!(
                                            "[MediaUploadZone] Using on_file_selected callback"
                                        );
                                        let mut file_vec = Vec::new();
                                        let limit = if max_files > 0 {
                                            max_files.min(files.length() as usize)
                                        } else {
                                            files.length() as usize
                                        };

                                        for i in 0..limit {
                                            if let Some(file) = files.get(i as u32) {
                                                // Validate file type
                                                if validate_file_type(&file, &allowed_refs) {
                                                    file_vec.push(file);
                                                } else {
                                                    gloo_console::warn!(
                                                        "[MediaUploadZone] File type not allowed:",
                                                        file.type_()
                                                    );
                                                }
                                            }
                                        }

                                        if !file_vec.is_empty() {
                                            on_file_selected.call(file_vec);
                                        }
                                    } else {
                                        // Original behavior: automatic upload
                                        let blob_urls = process_files_async(
                                            files,
                                            max_files,
                                            &allowed_refs,
                                            reference_type_clone,
                                        )
                                        .await;

                                        gloo_console::log!(
                                            "[MediaUploadZone] Processing complete, blob URLs: ",
                                            blob_urls.len().to_string()
                                        );

                                        if !blob_urls.is_empty() {
                                            on_upload_clone.call(blob_urls);
                                        }
                                    }

                                    // Reset input
                                    target.set_value("");
                                }
                            } else {
                                gloo_console::warn!("[MediaUploadZone] No files found in input");
                            }
                        }
                    }
                }
            }
        });
    };

    // TODO: Add drag-and-drop support
    // Drag event handlers are temporarily disabled due to type compatibility issues
    // Will be re-enabled after fixing event handler signatures

    let border_class =
        "border-zinc-200 dark:border-zinc-800 hover:border-zinc-300 dark:hover:border-zinc-700";

    rsx! {
        div {
            class: "border-2 border-dashed rounded-lg p-8 text-center transition-colors {border_class}",

            // Hidden file input
            input {
                r#type: "file",
                accept: if props.allowed_types.is_empty() { "*".to_string() } else { props.allowed_types.join(",") },
                multiple: props.multiple,
                class: "hidden",
                id: "{input_id()}",
                onchange: handle_file_change,
            }

            div { class: "flex flex-col items-center gap-3",
                // Icon
                div { class: "p-4 rounded-full bg-zinc-100 dark:bg-zinc-800",
                    Icon { icon: LdUpload, class: "h-8 w-8 text-zinc-500 dark:text-zinc-400" }
                }

                // Text
                div { class: "space-y-1",
                    p { class: "text-sm font-medium text-foreground",
                        "{props.title}"
                    }
                    p { class: "text-xs text-muted-foreground",
                        "{props.description}"
                    }
                    if props.max_files > 0 {
                        {
                            let file_text = if props.max_files > 1 { "files" } else { "file" };
                            rsx! {
                                p { class: "text-xs text-muted-foreground",
                                    "Maximum {props.max_files} {file_text}"
                                }
                            }
                        }
                    }
                }

                // Button
                Button {
                    r#type: "button".to_string(),
                    variant: ButtonVariant::Outline,
                    onclick: open_picker,
                    class: "border-zinc-200 dark:border-zinc-800".to_string(),
                    "Select Files"
                }
            }
        }
    }
}

/// Process selected files and initiate uploads
/// Returns vector of blob URLs
async fn process_files_async(
    files: web_sys::FileList,
    max_files: usize,
    allowed_types: &[&str],
    reference_type: Option<MediaReference>,
) -> Vec<String> {
    gloo_console::log!("[process_files_async] Starting file processing");

    let media_state = use_media();
    let mut blob_urls = Vec::new();
    let limit = if max_files > 0 {
        max_files.min(files.length() as usize)
    } else {
        files.length() as usize
    };

    gloo_console::log!(
        "[process_files_async] Processing limit: ",
        limit.to_string(),
        " files"
    );

    for i in 0..limit {
        if let Some(file) = files.get(i as u32) {
            let filename = file.name();
            let file_type = file.type_();
            let file_size = file.size();

            gloo_console::log!(
                "[process_files_async] File ",
                i.to_string(),
                " - Name: ",
                &filename,
                " Type: ",
                &file_type,
                " Size: ",
                file_size.to_string()
            );

            // Validate file type
            if !validate_file_type(&file, allowed_types) {
                gloo_console::warn!(
                    "[process_files_async] File type not allowed:",
                    &file_type,
                    "- Allowed:",
                    format!("{:?}", allowed_types)
                );
                continue;
            }

            gloo_console::log!("[process_files_async] File type validated, creating payload");

            // Create payload
            let payload = MediaUploadPayload {
                file: file.clone(),
                reference_type: reference_type.clone(),
                width: None, // Could be calculated if needed
                height: None,
            };

            // Initiate upload (returns blob URL immediately)
            gloo_console::log!("[process_files_async] Initiating upload for:", &filename);
            match media_state.upload(payload).await {
                Ok(blob_url) => {
                    gloo_console::log!(
                        "[process_files_async] Upload initiated successfully, blob URL:",
                        &blob_url
                    );
                    blob_urls.push(blob_url);
                }
                Err(e) => {
                    gloo_console::error!(
                        "[process_files_async] Upload failed for",
                        &filename,
                        ":",
                        e
                    );
                }
            }
        } else {
            gloo_console::warn!(
                "[process_files_async] Could not get file at index: ",
                i.to_string()
            );
        }
    }

    gloo_console::log!(
        "[process_files_async] Processing complete, total blob URLs: ",
        blob_urls.len().to_string()
    );
    blob_urls
}
