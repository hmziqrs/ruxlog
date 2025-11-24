use super::{LikeActionResponse, LikeState, LikeStatus, LikeStatusBatchRequest, LikeStatusBatchResponse};
use oxcore::http;
use oxstore::StateFrame;

impl LikeState {
    /// Like a post
    pub async fn like(&self, post_id: i32) {
        // Set loading state
        {
            let mut action_map = self.action.write();
            action_map
                .entry(post_id)
                .or_insert_with(StateFrame::new)
                .set_loading();
        }

        let result = http::post(&format!("/post/v1/like/{}", post_id), &())
            .send()
            .await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let body_text = response.body_text();
                    match response.json::<LikeActionResponse>().await {
                        Ok(data) => {
                            // Update action state
                            {
                                let mut action_map = self.action.write();
                                action_map
                                    .entry(post_id)
                                    .or_insert_with(StateFrame::new)
                                    .set_success(Some(data.clone()));
                            }
                            // Update status cache
                            {
                                let mut status_map = self.status.write();
                                status_map
                                    .entry(post_id)
                                    .or_insert_with(StateFrame::new)
                                    .set_success(Some(LikeStatus {
                                        post_id: data.post_id,
                                        is_liked: data.is_liked,
                                        likes_count: data.likes_count,
                                    }));
                            }
                        }
                        Err(e) => {
                            dioxus::logger::tracing::error!(
                                "Failed to parse like response: {:?}\nResponse: {}",
                                e,
                                body_text
                            );
                            let mut action_map = self.action.write();
                            action_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_decode_error("like", format!("{}", e), Some(body_text));
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let mut action_map = self.action.write();
                    action_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                let mut action_map = self.action.write();
                action_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    /// Unlike a post
    pub async fn unlike(&self, post_id: i32) {
        // Set loading state
        {
            let mut action_map = self.action.write();
            action_map
                .entry(post_id)
                .or_insert_with(StateFrame::new)
                .set_loading();
        }

        let result = http::post(&format!("/post/v1/unlike/{}", post_id), &())
            .send()
            .await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let body_text = response.body_text();
                    match response.json::<LikeActionResponse>().await {
                        Ok(data) => {
                            // Update action state
                            {
                                let mut action_map = self.action.write();
                                action_map
                                    .entry(post_id)
                                    .or_insert_with(StateFrame::new)
                                    .set_success(Some(data.clone()));
                            }
                            // Update status cache
                            {
                                let mut status_map = self.status.write();
                                status_map
                                    .entry(post_id)
                                    .or_insert_with(StateFrame::new)
                                    .set_success(Some(LikeStatus {
                                        post_id: data.post_id,
                                        is_liked: data.is_liked,
                                        likes_count: data.likes_count,
                                    }));
                            }
                        }
                        Err(e) => {
                            dioxus::logger::tracing::error!(
                                "Failed to parse unlike response: {:?}\nResponse: {}",
                                e,
                                body_text
                            );
                            let mut action_map = self.action.write();
                            action_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_decode_error("unlike", format!("{}", e), Some(body_text));
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let mut action_map = self.action.write();
                    action_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                let mut action_map = self.action.write();
                action_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    /// Toggle like/unlike for a post
    pub async fn toggle(&self, post_id: i32) {
        if self.is_liked(post_id) {
            self.unlike(post_id).await;
        } else {
            self.like(post_id).await;
        }
    }

    /// Fetch like status for a single post
    pub async fn fetch_status(&self, post_id: i32) {
        // Set loading state
        {
            let mut status_map = self.status.write();
            status_map
                .entry(post_id)
                .or_insert_with(StateFrame::new)
                .set_loading();
        }

        let result = http::post(&format!("/post/v1/like/status/{}", post_id), &())
            .send()
            .await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let body_text = response.body_text();
                    match response.json::<LikeStatus>().await {
                        Ok(data) => {
                            let mut status_map = self.status.write();
                            status_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_success(Some(data));
                        }
                        Err(e) => {
                            dioxus::logger::tracing::error!(
                                "Failed to parse like status: {:?}\nResponse: {}",
                                e,
                                body_text
                            );
                            let mut status_map = self.status.write();
                            status_map
                                .entry(post_id)
                                .or_insert_with(StateFrame::new)
                                .set_decode_error("like_status", format!("{}", e), Some(body_text));
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let mut status_map = self.status.write();
                    status_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                let mut status_map = self.status.write();
                status_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    /// Fetch like status for multiple posts at once
    pub async fn fetch_status_batch(&self, post_ids: Vec<i32>) {
        if post_ids.is_empty() {
            return;
        }

        // Set loading state for all posts
        {
            let mut status_map = self.status.write();
            for &post_id in &post_ids {
                status_map
                    .entry(post_id)
                    .or_insert_with(StateFrame::new)
                    .set_loading();
            }
        }

        let payload = LikeStatusBatchRequest { post_ids: post_ids.clone() };
        let result = http::post("/post/v1/like/status/batch", &payload)
            .send()
            .await;

        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let body_text = response.body_text();
                    match response.json::<LikeStatusBatchResponse>().await {
                        Ok(data) => {
                            let mut status_map = self.status.write();
                            for status in data.statuses {
                                status_map
                                    .entry(status.post_id)
                                    .or_insert_with(StateFrame::new)
                                    .set_success(Some(status));
                            }
                        }
                        Err(e) => {
                            dioxus::logger::tracing::error!(
                                "Failed to parse batch like status: {:?}\nResponse: {}",
                                e,
                                body_text
                            );
                            // Set error for all requested posts
                            let mut status_map = self.status.write();
                            for post_id in post_ids {
                                status_map
                                    .entry(post_id)
                                    .or_insert_with(StateFrame::new)
                                    .set_decode_error(
                                        "like_status_batch",
                                        format!("{}", e),
                                        Some(body_text.clone()),
                                    );
                            }
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let mut status_map = self.status.write();
                    for post_id in post_ids {
                        status_map
                            .entry(post_id)
                            .or_insert_with(StateFrame::new)
                            .set_api_error(status, body.clone());
                    }
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                let mut status_map = self.status.write();
                for post_id in post_ids {
                    status_map
                        .entry(post_id)
                        .or_insert_with(StateFrame::new)
                        .set_transport_error(kind.clone(), Some(msg.clone()));
                }
            }
        }
    }
}
