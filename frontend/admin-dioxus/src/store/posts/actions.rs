use super::{
    Post, PostCreatePayload, PostEditPayload, PostListQuery, PostRevision, PostSchedulePayload,
    PostState, Series, SeriesCreatePayload, SeriesEditPayload, SeriesListQuery,
};
use crate::services::http_client;
use crate::store::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction, view_state_abstraction, PaginatedList, StateFrame,
};
use std::collections::HashMap;

// ============================================================================
// Core Post CRUD Operations
// ============================================================================

impl PostState {
    /// Create a new post
    pub async fn add(&self, payload: PostCreatePayload) {
        let meta_payload = payload.clone();
        let request = http_client::post("/post/v1/create", &payload);
        let created = state_request_abstraction(
            &self.add,
            Some(meta_payload),
            request.send(),
            "post",
            |post: &Post| (Some(post.clone()), None),
        )
        .await;

        if created.is_some() {
            self.list().await;
        }
    }

    /// Update an existing post
    pub async fn edit(&self, post_id: i32, payload: PostEditPayload) {
        let _post = edit_state_abstraction(
            &self.edit,
            post_id,
            payload.clone(),
            http_client::post(&format!("/post/v1/update/{}", post_id), &payload).send(),
            "post",
            Some(&self.list),
            Some(&self.view),
            |post: &Post| post.id,
            None::<fn(&Post)>,
        )
        .await;
    }

    /// Delete a post
    pub async fn remove(&self, post_id: i32) {
        let _ = remove_state_abstraction(
            &self.remove,
            post_id,
            http_client::post(&format!("/post/v1/delete/{}", post_id), &()).send(),
            "post",
            Some(&self.list),
            Some(&self.view),
            |post: &Post| post.id,
            None::<fn()>,
        )
        .await;
    }

    /// List posts with default query
    pub async fn list(&self) {
        let _ = list_state_abstraction::<PaginatedList<Post>>(
            &self.list,
            http_client::post("/post/v1/query", &serde_json::json!({})),
            "posts",
        )
        .await;
    }

    /// List posts with custom query parameters
    pub async fn list_with_query(&self, query: PostListQuery) {
        let _ = list_state_abstraction::<PaginatedList<Post>>(
            &self.list,
            http_client::post("/post/v1/query", &query),
            "posts",
        )
        .await;
    }

    /// View a single post by ID or slug
    /// Note: This method fetches by id_or_slug but caches by post.id
    pub async fn view(&self, id_or_slug: &str) {
        // We need to handle this manually since the key might be a slug but we cache by id
        let result = http_client::post(&format!("/post/v1/view/{}", id_or_slug), &())
            .send()
            .await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    match response.json::<Post>().await {
                        Ok(post) => {
                            let post_id = post.id;
                            let mut view_map = self.view.write();
                            view_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_success(Some(post));
                        }
                        Err(e) => {
                            // Can't cache without knowing the ID, just log the error
                            dioxus::logger::tracing::error!("Failed to parse post: {}", e);
                        }
                    }
                } else {
                    dioxus::logger::tracing::error!(
                        "Failed to fetch post: status {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                dioxus::logger::tracing::error!("Network error fetching post: {}", e);
            }
        }
    }

    /// View a post by ID (preferred when ID is known)
    pub async fn view_by_id(&self, post_id: i32) {
        let _ = view_state_abstraction(
            &self.view,
            post_id,
            http_client::post(&format!("/post/v1/view/{}", post_id), &()).send(),
            "post",
            |post: &Post| post.clone(),
        )
        .await;
    }

    /// List published posts (public endpoint)
    pub async fn list_published(&self) {
        let _ = list_state_abstraction::<PaginatedList<Post>>(
            &self.list,
            http_client::post("/post/v1/list/published", &serde_json::json!({})),
            "published posts",
        )
        .await;
    }

    // ============================================================================
    // Post Scheduling
    // ============================================================================

    /// Schedule a post for future publication
    pub async fn schedule(&self, payload: PostSchedulePayload) {
        let post_id = payload.post_id;
        let mut schedule_map = self.schedule.write();
        schedule_map
            .entry(post_id)
            .or_insert_with(StateFrame::new)
            .set_loading();
        drop(schedule_map);

        let result = http_client::post("/post/v1/schedule", &payload)
            .send()
            .await;

        let mut schedule_map = self.schedule.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    schedule_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);

                    // Refresh the post view and list
                    drop(schedule_map);
                    self.view_by_id(post_id).await;
                    self.list().await;
                } else {
                    schedule_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(&response)
                        .await;
                }
            }
            Err(e) => {
                let (kind, msg) = crate::store::classify_transport_error(&e);
                schedule_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    // ============================================================================
    // Post Revisions
    // ============================================================================

    /// List all revisions for a post
    pub async fn revisions_list(&self, post_id: i32) {
        let mut revisions_map = self.revisions_list.write();
        revisions_map
            .entry(post_id)
            .or_insert_with(StateFrame::new)
            .set_loading();
        drop(revisions_map);

        let result = http_client::post(
            &format!("/post/v1/revisions/{}/list", post_id),
            &serde_json::json!({}),
        )
        .send()
        .await;

        let mut revisions_map = self.revisions_list.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    match response.json::<Vec<PostRevision>>().await {
                        Ok(revisions) => {
                            revisions_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_success(Some(revisions));
                        }
                        Err(e) => {
                            let raw = response.text().await.unwrap_or_default();
                            revisions_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_decode_error("revisions", format!("{}", e), Some(raw));
                        }
                    }
                } else {
                    revisions_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(&response)
                        .await;
                }
            }
            Err(e) => {
                let (kind, msg) = crate::store::classify_transport_error(&e);
                revisions_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    /// Restore a specific revision of a post
    pub async fn revisions_restore(&self, post_id: i32, revision_id: i32) {
        let key = (post_id, revision_id);
        let mut restore_map = self.revisions_restore.write();
        restore_map
            .entry(key)
            .or_insert_with(StateFrame::new)
            .set_loading();
        drop(restore_map);

        let result = http_client::post(
            &format!("/post/v1/revisions/{}/restore/{}", post_id, revision_id),
            &serde_json::json!({}),
        )
        .send()
        .await;

        let mut restore_map = self.revisions_restore.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    restore_map
                        .entry(key)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);

                    // Refresh the post view
                    drop(restore_map);
                    self.view_by_id(post_id).await;
                    self.list().await;
                } else {
                    restore_map
                        .entry(key)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(&response)
                        .await;
                }
            }
            Err(e) => {
                let (kind, msg) = crate::store::classify_transport_error(&e);
                restore_map
                    .entry(key)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    // ============================================================================
    // View Tracking
    // ============================================================================

    /// Track a view for a post (public endpoint)
    pub async fn track_view(&self, post_id: i32) {
        let mut track_map = self.track_view.write();
        track_map
            .entry(post_id)
            .or_insert_with(StateFrame::new)
            .set_loading();
        drop(track_map);

        let result = http_client::post(&format!("/post/v1/track_view/{}", post_id), &())
            .send()
            .await;

        let mut track_map = self.track_view.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    track_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);
                } else {
                    track_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(&response)
                        .await;
                }
            }
            Err(e) => {
                let (kind, msg) = crate::store::classify_transport_error(&e);
                track_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    // ============================================================================
    // Series Management
    // ============================================================================

    /// Create a new series
    pub async fn series_create(&self, payload: SeriesCreatePayload) {
        let meta_payload = payload.clone();
        let request = http_client::post("/post/v1/series/create", &payload);
        let created = state_request_abstraction(
            &self.series_add,
            Some(meta_payload),
            request.send(),
            "series",
            |_series: &Series| (None, None),
        )
        .await;

        if created.is_some() {
            self.series_list().await;
        }
    }

    /// Update an existing series
    pub async fn series_update(&self, series_id: i32, payload: SeriesEditPayload) {
        let _series = edit_state_abstraction(
            &self.series_edit,
            series_id,
            payload.clone(),
            http_client::post(&format!("/post/v1/series/update/{}", series_id), &payload).send(),
            "series",
            Some(&self.series_list),
            Some(&self.series_view),
            |series: &Series| series.id,
            None::<fn(&Series)>,
        )
        .await;
    }

    /// Delete a series
    pub async fn series_delete(&self, series_id: i32) {
        let _ = remove_state_abstraction(
            &self.series_remove,
            series_id,
            http_client::post(&format!("/post/v1/series/delete/{}", series_id), &()).send(),
            "series",
            Some(&self.series_list),
            Some(&self.series_view),
            |series: &Series| series.id,
            None::<fn()>,
        )
        .await;
    }

    /// List all series
    pub async fn series_list(&self) {
        let _ = list_state_abstraction::<PaginatedList<Series>>(
            &self.series_list,
            http_client::post("/post/v1/series/list", &serde_json::json!({})),
            "series",
        )
        .await;
    }

    /// List series with query parameters
    pub async fn series_list_with_query(&self, query: SeriesListQuery) {
        let _ = list_state_abstraction::<PaginatedList<Series>>(
            &self.series_list,
            http_client::post("/post/v1/series/list", &query),
            "series",
        )
        .await;
    }

    /// Add a post to a series
    pub async fn series_add_post(&self, post_id: i32, series_id: i32) {
        let key = (post_id, series_id);
        let mut add_map = self.series_add_post.write();
        add_map
            .entry(key)
            .or_insert_with(StateFrame::new)
            .set_loading();
        drop(add_map);

        let result = http_client::post(
            &format!("/post/v1/series/add/{}/{}", post_id, series_id),
            &serde_json::json!({}),
        )
        .send()
        .await;

        let mut add_map = self.series_add_post.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    add_map
                        .entry(key)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);

                    // Refresh the post view
                    drop(add_map);
                    self.view_by_id(post_id).await;
                } else {
                    add_map
                        .entry(key)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(&response)
                        .await;
                }
            }
            Err(e) => {
                let (kind, msg) = crate::store::classify_transport_error(&e);
                add_map
                    .entry(key)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    /// Remove a post from a series
    pub async fn series_remove_post(&self, post_id: i32, series_id: i32) {
        let key = (post_id, series_id);
        let mut remove_map = self.series_remove_post.write();
        remove_map
            .entry(key)
            .or_insert_with(StateFrame::new)
            .set_loading();
        drop(remove_map);

        let result = http_client::post(
            &format!("/post/v1/series/remove/{}/{}", post_id, series_id),
            &serde_json::json!({}),
        )
        .send()
        .await;

        let mut remove_map = self.series_remove_post.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    remove_map
                        .entry(key)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);

                    // Refresh the post view
                    drop(remove_map);
                    self.view_by_id(post_id).await;
                } else {
                    remove_map
                        .entry(key)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(&response)
                        .await;
                }
            }
            Err(e) => {
                let (kind, msg) = crate::store::classify_transport_error(&e);
                remove_map
                    .entry(key)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    // ============================================================================
    // Sitemap (Public)
    // ============================================================================

    /// Get sitemap data for published posts
    pub async fn sitemap(&self) -> Option<Vec<Post>> {
        let result = http_client::post("/post/v1/sitemap", &serde_json::json!({}))
            .send()
            .await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    response.json::<Vec<Post>>().await.ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    // ============================================================================
    // State Reset
    // ============================================================================

    /// Reset all post state to initial values
    pub fn reset(&self) {
        *self.view.write() = HashMap::new();
        *self.list.write() = StateFrame::new();
        *self.add.write() = StateFrame::new();
        *self.edit.write() = HashMap::new();
        *self.remove.write() = HashMap::new();
        *self.schedule.write() = HashMap::new();
        *self.revisions_list.write() = HashMap::new();
        *self.revisions_restore.write() = HashMap::new();
        *self.track_view.write() = HashMap::new();
        *self.series_list.write() = StateFrame::new();
        *self.series_view.write() = HashMap::new();
        *self.series_add.write() = StateFrame::new();
        *self.series_edit.write() = HashMap::new();
        *self.series_remove.write() = HashMap::new();
        *self.series_add_post.write() = HashMap::new();
        *self.series_remove_post.write() = HashMap::new();
    }
}

// ============================================================================
// ListStore Trait Implementation
// ============================================================================

use crate::store::ListStore;
use dioxus::prelude::GlobalSignal;

impl ListStore<Post, PostListQuery> for PostState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<Post>>> {
        &self.list
    }

    async fn fetch_list(&self) {
        self.list().await;
    }

    async fn fetch_list_with_query(&self, query: PostListQuery) {
        self.list_with_query(query).await;
    }
}
