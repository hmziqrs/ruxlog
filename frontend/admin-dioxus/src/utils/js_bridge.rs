use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::File;

/// JavaScript bridge for Editor.js media uploads
/// Exposes window.editorjs_upload_file() to JavaScript
#[wasm_bindgen]
pub async fn editorjs_upload_file(file: File) -> Result<JsValue, JsValue> {
    use crate::store::{use_media, MediaReference, MediaUploadPayload, UploadStatus};
    use serde::Serialize;

    gloo_console::log!("[editorjs_upload_file] Starting upload for:", file.name());

    // Get media store reference (this is fine, it's just a static reference)
    let media_store = use_media();

    // Create upload payload
    let payload = MediaUploadPayload {
        file,
        reference_type: Some(MediaReference::Post),
        width: None,
        height: None,
    };

    // Upload via media store
    match media_store.upload(payload).await {
        Ok(blob_url) => {
            gloo_console::log!(
                "[editorjs_upload_file] Upload initiated, blob URL:",
                &blob_url
            );

            // Poll for upload completion
            let max_wait = 30; // 30 seconds timeout
            let mut elapsed = 0;

            loop {
                if elapsed >= max_wait {
                    let err_msg = "Upload timeout after 30 seconds";
                    gloo_console::error!("[editorjs_upload_file]", err_msg);
                    return Err(JsValue::from_str(err_msg));
                }

                if media_store.is_upload_complete(&blob_url) {
                    match media_store.get_uploaded_media(&blob_url) {
                        Some(media) => {
                            gloo_console::log!(
                                "[editorjs_upload_file] Upload complete! Media ID:",
                                media.id.to_string(),
                                "URL:",
                                &media.file_url
                            );

                            // Return Editor.js compatible format
                            #[derive(Serialize)]
                            struct EditorJsUploadResponse {
                                success: u8,
                                file: EditorJsFile,
                            }

                            #[derive(Serialize)]
                            struct EditorJsFile {
                                url: String,
                                media_id: i32,
                            }

                            let response = EditorJsUploadResponse {
                                success: 1,
                                file: EditorJsFile {
                                    url: media.file_url,
                                    media_id: media.id,
                                },
                            };

                            // Cleanup blob tracking
                            media_store.cleanup_blob(&blob_url);

                            return serde_wasm_bindgen::to_value(&response).map_err(|e| {
                                JsValue::from_str(&format!("Serialization error: {:?}", e))
                            });
                        }
                        None => {
                            // Check if there was an error
                            if let Some(status) = media_store.get_upload_status(&blob_url) {
                                if let UploadStatus::Error(err_msg) = status {
                                    gloo_console::error!(
                                        "[editorjs_upload_file] Upload failed:",
                                        &err_msg
                                    );
                                    media_store.cleanup_blob(&blob_url);
                                    return Err(JsValue::from_str(&err_msg));
                                }
                            }
                        }
                    }
                }

                // Wait 500ms before checking again
                gloo_timers::future::TimeoutFuture::new(500).await;
                elapsed += 1;
            }
        }
        Err(err_msg) => {
            gloo_console::error!("[editorjs_upload_file] Upload failed:", &err_msg);
            Err(JsValue::from_str(&err_msg))
        }
    }
}
