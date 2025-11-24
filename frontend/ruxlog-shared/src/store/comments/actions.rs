use super::{
    Comment, CommentCreatePayload, CommentFlagPayload, CommentFlagSummary, CommentListQuery,
    CommentState, CommentUpdatePayload,
};
use dioxus::prelude::ReadableExt;
use oxcore::http;
use oxstore::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction, view_state_abstraction, StateFrame,
};

impl CommentState {
    fn cache_comment(&self, comment: Comment) {
        self.view
            .write()
            .entry(comment.id)
            .or_insert_with(StateFrame::new)
            .set_success(Some(comment));
    }

    /// Create a new comment
    pub async fn create(&self, payload: CommentCreatePayload) {
        let meta = payload.clone();
        let created = state_request_abstraction(
            &self.add,
            Some(meta),
            http::post("/post/comment/v1/create", &payload).send(),
            "comment",
            |comment: &Comment| (Some(comment.clone()), None),
        )
        .await;

        if let Some(comment) = created {
            self.cache_comment(comment.clone());
            self.list(comment.post_id).await;
        }
    }

    /// Update an existing comment
    pub async fn update(&self, comment_id: i32, payload: CommentUpdatePayload) {
        let updated_comment =
            edit_state_abstraction::<i32, Comment, CommentUpdatePayload, _, _, fn(&Comment)>(
                &self.edit,
                comment_id,
                payload.clone(),
                http::post(&format!("/post/comment/v1/update/{}", comment_id), &payload).send(),
                "comment",
                None,
                Some(&self.view),
                |comment: &Comment| comment.id,
                None,
            )
            .await;

        if let Some(comment) = updated_comment {
            self.cache_comment(comment.clone());
            // Refresh the list using the known post_id
            self.list(comment.post_id).await;
        }
    }

    /// Remove a comment (user or admin)
    pub async fn remove(&self, comment_id: i32) {
        // First, get the post_id before removing
        let post_id = {
            let view_map = self.view.read();
            view_map
                .get(&comment_id)
                .and_then(|frame| frame.data.as_ref())
                .map(|comment| comment.post_id)
        };

        let _removed = remove_state_abstraction::<i32, Comment, _, _, fn()>(
            &self.remove,
            comment_id,
            http::post(&format!("/post/comment/v1/delete/{}", comment_id), &()).send(),
            "comment",
            None,
            Some(&self.view),
            |comment: &Comment| comment.id,
            None,
        )
        .await;

        // Refresh list if we know the post_id
        if let Some(post_id) = post_id {
            self.list(post_id).await;
        }
    }

    /// Flag a comment
    pub async fn flag(&self, comment_id: i32, payload: CommentFlagPayload) {
        {
            let mut map = self.flag_actions.write();
            map.entry(comment_id)
                .or_insert_with(StateFrame::new)
                .set_loading();
        }

        let result = http::post(&format!("/post/comment/v1/flag/{}", comment_id), &payload)
            .send()
            .await;

        let mut map = self.flag_actions.write();
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    map.entry(comment_id)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    map.entry(comment_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                map.entry(comment_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    /// List comments for a post
    pub async fn list(&self, post_id: i32) {
        let _ = list_state_abstraction(
            &self.list,
            http::post(
                &format!("/post/comment/v1/{}", post_id),
                &serde_json::json!({}),
            )
            .send(),
            "comments",
        )
        .await;
    }

    /// Admin list with filters
    pub async fn admin_list(&self, query: CommentListQuery) {
        let _ = list_state_abstraction(
            &self.list,
            http::post("/post/comment/v1/admin/list", &query).send(),
            "comments",
        )
        .await;
    }

    /// Hide a comment (admin)
    pub async fn hide(&self, comment_id: i32) {
        self.run_moderation_action(
            comment_id,
            format!("/post/comment/v1/admin/hide/{}", comment_id),
        )
        .await;
    }

    /// Unhide a comment (admin)
    pub async fn unhide(&self, comment_id: i32) {
        self.run_moderation_action(
            comment_id,
            format!("/post/comment/v1/admin/unhide/{}", comment_id),
        )
        .await;
    }

    /// Delete a comment via the admin endpoint
    pub async fn delete_admin(&self, comment_id: i32) {
        self.run_moderation_action(
            comment_id,
            format!("/post/comment/v1/admin/delete/{}", comment_id),
        )
        .await;
    }

    /// View all flags across comments
    pub async fn list_flags(&self) {
        let _ = list_state_abstraction(
            &self.flags,
            http::post("/post/comment/v1/admin/flags/list", &serde_json::json!({})).send(),
            "flags",
        )
        .await;
    }

    /// Clear flags for a specific comment
    pub async fn clear_flags(&self, comment_id: i32) {
        self.run_moderation_action(
            comment_id,
            format!("/post/comment/v1/admin/flags/clear/{}", comment_id),
        )
        .await;

        self.list_flags().await;
    }

    /// Fetch a flag summary for a single comment
    pub async fn flag_summary(&self, comment_id: i32) {
        let _ = view_state_abstraction(
            &self.summaries,
            comment_id,
            http::post(
                &format!("/post/comment/v1/admin/flags/summary/{}", comment_id),
                &(),
            )
            .send(),
            "comment_flag_summary",
            |summary: &CommentFlagSummary| summary.clone(),
        )
        .await;
    }

    async fn run_moderation_action(&self, comment_id: i32, url: String) {
        {
            let mut map = self.moderation.write();
            map.entry(comment_id)
                .or_insert_with(StateFrame::new)
                .set_loading();
        }

        let result = http::post(&url, &()).send().await;
        let mut map = self.moderation.write();

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    map.entry(comment_id)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    map.entry(comment_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                map.entry(comment_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }
}
