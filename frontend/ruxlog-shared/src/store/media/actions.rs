use super::{
    Media, MediaListQuery, MediaState, MediaUploadPayload, MediaUsageDetails,
    MediaUsageDetailsRequest, MediaUsageDetailsResponse, UploadStatus,
};
use oxcore::http;

use oxstore::{
    list_state_abstraction, remove_state_abstraction, view_state_abstraction, StateFrame,
};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, FormData, Url};

impl MediaState {
    #[cfg(target_arch = "wasm32")]
    /// Hybrid upload: returns blob URL immediately, uploads in background
    pub async fn upload(&self, payload: MediaUploadPayload) -> Result<String, String> {
        dioxus::logger::tracing::debug!("[MediaState::upload] Starting upload");

        // 1. Create blob URL immediately for instant preview
        let blob: &Blob = payload.file.as_ref();
        dioxus::logger::tracing::debug!("[MediaState::upload] Creating blob URL for file");

        let blob_url = Url::create_object_url_with_blob(blob).map_err(|e| {
            let err_msg = format!("Failed to create blob URL: {:?}", e);
            dioxus::logger::tracing::error!("[MediaState::upload] {}", &err_msg);
            err_msg
        })?;

        // Extract file info
        let filename = payload.file.name();
        let size = payload.file.size() as i64;
        let mime_type = payload.file.type_();

        dioxus::logger::tracing::debug!(
            "[MediaState::upload] Blob URL created: {} | File: {} | Size: {} | Type: {}",
            &blob_url,
            &filename,
            size,
            &mime_type
        );

        // 2. Initialize tracking state
        dioxus::logger::tracing::debug!("[MediaState::upload] Initializing tracking state");
        {
            let mut status_map = self.upload_status.write();
            status_map.insert(blob_url.clone(), UploadStatus::Uploading);
        }
        {
            let mut progress_map = self.upload_progress.write();
            progress_map.insert(blob_url.clone(), 0.0);
        }
        {
            let mut blob_map = self.blob_to_media.write();
            blob_map.insert(blob_url.clone(), None);
        }
        {
            let mut file_info_map = self.blob_file_info.write();
            file_info_map.insert(
                blob_url.clone(),
                super::FileInfo {
                    filename: filename.clone(),
                    size,
                },
            );
        }
        dioxus::logger::tracing::debug!("[MediaState::upload] Tracking state initialized");

        // 3. Prepare multipart form data
        dioxus::logger::tracing::debug!("[MediaState::upload] Preparing form data");
        let form_data = FormData::new().map_err(|e| {
            let err_msg = format!("Failed to create FormData: {:?}", e);
            dioxus::logger::tracing::error!("[MediaState::upload] {}", &err_msg);
            err_msg
        })?;

        form_data
            .append_with_blob("file", &payload.file)
            .map_err(|e| {
                let err_msg = format!("Failed to append file: {:?}", e);
                dioxus::logger::tracing::error!("[MediaState::upload] {}", &err_msg);
                err_msg
            })?;

        if let Some(ref_type) = &payload.reference_type {
            dioxus::logger::tracing::debug!(
                "[MediaState::upload] Adding reference_type: {}",
                ref_type.to_string()
            );
            form_data
                .append_with_str("reference_type", &ref_type.to_string())
                .map_err(|e| {
                    let err_msg = format!("Failed to append reference_type: {:?}", e);
                    dioxus::logger::tracing::error!("[MediaState::upload] {}", &err_msg);
                    err_msg
                })?;
        }

        if let Some(width) = payload.width {
            dioxus::logger::tracing::debug!("[MediaState::upload] Adding width: {}", width);
            form_data
                .append_with_str("width", &width.to_string())
                .map_err(|e| {
                    let err_msg = format!("Failed to append width: {:?}", e);
                    dioxus::logger::tracing::error!("[MediaState::upload] {}", &err_msg);
                    err_msg
                })?;
        }

        if let Some(height) = payload.height {
            dioxus::logger::tracing::debug!("[MediaState::upload] Adding height: {}", height);
            form_data
                .append_with_str("height", &height.to_string())
                .map_err(|e| {
                    let err_msg = format!("Failed to append height: {:?}", e);
                    dioxus::logger::tracing::error!("[MediaState::upload] {}", &err_msg);
                    err_msg
                })?;
        }

        dioxus::logger::tracing::debug!("[MediaState::upload] Form data prepared successfully");

        // 4. Upload in background
        let blob_url_clone = blob_url.clone();
        dioxus::logger::tracing::debug!(
            "[MediaState::upload] Spawning background upload task for: {}",
            &filename
        );

        wasm_bindgen_futures::spawn_local(async move {
            use super::use_media;
            let media_state = use_media();

            dioxus::logger::tracing::debug!(
                "[MediaState::upload background] Creating HTTP request"
            );

            match http::post_multipart("/media/v1/create", &form_data) {
                Ok(request) => {
                    dioxus::logger::tracing::debug!(
                        "[MediaState::upload background] Request created, sending..."
                    );

                    match request.send().await {
                        Ok(response) => {
                            let status = response.status();
                            let is_ok = (200..300).contains(&status);
                            dioxus::logger::tracing::debug!(
                                "[MediaState::upload background] Response received - Status: {} OK: {}",
                                status,
                                is_ok
                            );

                            if is_ok {
                                dioxus::logger::tracing::debug!(
                                    "[MediaState::upload background] Parsing JSON response"
                                );

                                match response.json::<Media>().await {
                                    Ok(media) => {
                                        dioxus::logger::tracing::debug!(
                                            "[MediaState::upload background] Upload successful! Media ID: {} URL: {}",
                                            media.id,
                                            &media.file_url
                                        );

                                        // Success: update tracking
                                        {
                                            let mut status_map = media_state.upload_status.write();
                                            status_map.insert(
                                                blob_url_clone.clone(),
                                                UploadStatus::Success,
                                            );
                                        }
                                        {
                                            let mut progress_map =
                                                media_state.upload_progress.write();
                                            progress_map.insert(blob_url_clone.clone(), 100.0);
                                        }
                                        {
                                            let mut blob_map = media_state.blob_to_media.write();
                                            blob_map.insert(blob_url_clone.clone(), Some(media));
                                        }

                                        dioxus::logger::tracing::debug!("[MediaState::upload background] Status updated to Success");

                                        // Refresh list
                                        dioxus::logger::tracing::debug!(
                                            "[MediaState::upload background] Refreshing media list"
                                        );
                                        media_state.list().await;
                                    }
                                    Err(e) => {
                                        let err_msg = format!("Failed to parse response: {:?}", e);
                                        dioxus::logger::tracing::error!(
                                            "[MediaState::upload background] {}",
                                            &err_msg
                                        );

                                        let mut status_map = media_state.upload_status.write();
                                        status_map
                                            .insert(blob_url_clone, UploadStatus::Error(err_msg));
                                    }
                                }
                            } else {
                                let err_msg = format!("Upload failed with status: {}", status);
                                dioxus::logger::tracing::error!(
                                    "[MediaState::upload background] {}",
                                    &err_msg
                                );

                                let mut status_map = media_state.upload_status.write();
                                status_map.insert(blob_url_clone, UploadStatus::Error(err_msg));
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Request failed: {:?}", e);
                            dioxus::logger::tracing::error!(
                                "[MediaState::upload background] {}",
                                &err_msg
                            );

                            let mut status_map = media_state.upload_status.write();
                            status_map.insert(blob_url_clone, UploadStatus::Error(err_msg));
                        }
                    }
                }
                Err(e) => {
                    dioxus::logger::tracing::error!(
                        "[MediaState::upload background] Failed to create request: {}",
                        &e
                    );

                    let mut status_map = media_state.upload_status.write();
                    status_map.insert(blob_url_clone, UploadStatus::Error(e));
                }
            }
        });

        // 5. Return blob URL immediately
        dioxus::logger::tracing::debug!(
            "[MediaState::upload] Upload function complete, returning blob URL: {}",
            &blob_url
        );
        Ok(blob_url)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Native upload: not supported for native target
    pub async fn upload(&self, _payload: MediaUploadPayload) -> Result<String, String> {
        Err("File upload is only supported in WASM environment".to_string())
    }

    pub async fn remove(&self, id: i32) {
        let _ = remove_state_abstraction(
            &self.remove,
            id,
            http::post(&format!("/media/v1/delete/{}", id), &()).send(),
            "media",
            Some(&self.list),
            Some(&self.view),
            |media: &Media| media.id,
            None::<fn()>,
        )
        .await;
    }

    pub async fn list(&self) {
        let _ = list_state_abstraction(
            &self.list,
            http::post("/media/v1/list/query", &serde_json::json!({})).send(),
            "media",
        )
        .await;
    }

    pub async fn list_with_query(&self, query: MediaListQuery) {
        let _ = list_state_abstraction(
            &self.list,
            http::post("/media/v1/list/query", &query).send(),
            "media",
        )
        .await;
    }

    pub async fn view(&self, id: i32) {
        let _ = view_state_abstraction(
            &self.view,
            id,
            http::get(&format!("/media/v1/view/{}", id)).send(),
            "media",
            |media: &Media| media.clone(),
        )
        .await;
    }

    pub async fn usage_details(&self, id: i32) {
        let _ = view_state_abstraction(
            &self.usage_details,
            id,
            http::post(
                "/media/v1/usage/details",
                &MediaUsageDetailsRequest {
                    media_ids: vec![id],
                },
            )
            .send(),
            "media usage details",
            |response: &MediaUsageDetailsResponse| {
                response
                    .data
                    .first()
                    .cloned()
                    .unwrap_or_else(|| MediaUsageDetails {
                        media_id: id,
                        media: Media::default(),
                        posts: Vec::new(),
                        categories: Vec::new(),
                        users: Vec::new(),
                    })
            },
        )
        .await;
    }

    pub fn reset(&self) {
        *self.upload.write() = StateFrame::new();
        *self.remove.write() = HashMap::new();
        *self.list.write() = StateFrame::new();
        *self.view.write() = HashMap::new();
        *self.usage_details.write() = HashMap::new();
        *self.upload_progress.write() = HashMap::new();
        *self.upload_status.write() = HashMap::new();
        *self.blob_to_media.write() = HashMap::new();
        *self.blob_file_info.write() = HashMap::new();
    }

    // Helper methods for upload tracking

    /// Get the upload status for a blob URL
    pub fn get_upload_status(&self, blob_url: &str) -> Option<UploadStatus> {
        (*self.upload_status)().get(blob_url).cloned()
    }

    /// Get the uploaded media for a blob URL (if upload succeeded)
    pub fn get_uploaded_media(&self, blob_url: &str) -> Option<Media> {
        (*self.blob_to_media)()
            .get(blob_url)
            .and_then(|opt| opt.clone())
    }

    /// Get the upload progress percentage (0.0 - 100.0) for a blob URL
    pub fn get_upload_progress(&self, blob_url: &str) -> f64 {
        (*self.upload_progress)()
            .get(blob_url)
            .copied()
            .unwrap_or(0.0)
    }

    /// Check if an upload is complete (success or error)
    pub fn is_upload_complete(&self, blob_url: &str) -> bool {
        matches!(
            self.get_upload_status(blob_url),
            Some(UploadStatus::Success) | Some(UploadStatus::Error(_))
        )
    }

    /// Get the file info for a blob URL
    pub fn get_file_info(&self, blob_url: &str) -> Option<super::FileInfo> {
        (*self.blob_file_info)().get(blob_url).cloned()
    }

    #[cfg(target_arch = "wasm32")]
    /// Clean up tracking data for a blob URL (call after use)
    pub fn cleanup_blob(&self, blob_url: &str) {
        self.upload_progress.write().remove(blob_url);
        self.upload_status.write().remove(blob_url);
        self.blob_to_media.write().remove(blob_url);
        self.blob_file_info.write().remove(blob_url);

        // Revoke the blob URL to free memory
        Url::revoke_object_url(blob_url).ok();
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Clean up tracking data for a blob URL (call after use)
    pub fn cleanup_blob(&self, blob_url: &str) {
        self.upload_progress.write().remove(blob_url);
        self.upload_status.write().remove(blob_url);
        self.blob_to_media.write().remove(blob_url);
        self.blob_file_info.write().remove(blob_url);
        // No blob URL cleanup needed for native
    }
}
