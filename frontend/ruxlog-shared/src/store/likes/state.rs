use dioxus::prelude::*;
use oxstore::StateFrame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Response for like status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LikeStatus {
    pub post_id: i32,
    pub is_liked: bool,
    pub likes_count: i32,
}

/// Response for like/unlike action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LikeActionResponse {
    pub post_id: i32,
    pub is_liked: bool,
    pub likes_count: i32,
    pub message: String,
}

/// Request for batch like status check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeStatusBatchRequest {
    pub post_ids: Vec<i32>,
}

/// Response for batch like status check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeStatusBatchResponse {
    pub statuses: Vec<LikeStatus>,
}

/// State for managing post likes
pub struct LikeState {
    /// Like status for each post (keyed by post_id)
    pub status: GlobalSignal<HashMap<i32, StateFrame<LikeStatus>>>,
    /// Like action state (keyed by post_id)
    pub action: GlobalSignal<HashMap<i32, StateFrame<LikeActionResponse>>>,
}

impl LikeState {
    pub fn new() -> Self {
        Self {
            status: GlobalSignal::new(HashMap::new),
            action: GlobalSignal::new(HashMap::new),
        }
    }

    pub fn reset(&self) {
        *self.status.write() = HashMap::new();
        *self.action.write() = HashMap::new();
    }

    /// Get the current like status for a post from cache
    pub fn get_status(&self, post_id: i32) -> Option<LikeStatus> {
        let status_map = self.status.read();
        status_map
            .get(&post_id)
            .and_then(|frame| frame.data.clone())
    }

    /// Check if a post is liked (from cache)
    pub fn is_liked(&self, post_id: i32) -> bool {
        self.get_status(post_id)
            .map(|s| s.is_liked)
            .unwrap_or(false)
    }

    /// Get likes count for a post (from cache)
    pub fn likes_count(&self, post_id: i32) -> i32 {
        self.get_status(post_id)
            .map(|s| s.likes_count)
            .unwrap_or(0)
    }

    /// Check if an action is in progress for a post
    pub fn is_action_loading(&self, post_id: i32) -> bool {
        let action_map = self.action.read();
        action_map
            .get(&post_id)
            .map(|frame| frame.is_loading())
            .unwrap_or(false)
    }
}

impl Default for LikeState {
    fn default() -> Self {
        Self::new()
    }
}

static LIKE_STATE: OnceLock<LikeState> = OnceLock::new();

pub fn use_likes() -> &'static LikeState {
    LIKE_STATE.get_or_init(LikeState::new)
}
