use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{PaginatedList, StateFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentAuthor {
    pub id: i32,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub content: String,
    pub parent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub is_hidden: bool,
    #[serde(default)]
    pub author: Option<CommentAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentFlag {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentFlagSummary {
    pub comment_id: i32,
    pub total: u32,
    pub reasons: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentCreatePayload {
    pub post_id: i32,
    pub content: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CommentUpdatePayload {
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentFlagPayload {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CommentListQuery {
    pub post_id: Option<i32>,
    pub user_id: Option<i32>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub include_hidden: Option<bool>,
}

pub struct CommentState {
    pub list: GlobalSignal<StateFrame<PaginatedList<Comment>>>,
    pub view: GlobalSignal<HashMap<i32, StateFrame<Comment>>>,
    pub add: GlobalSignal<StateFrame<Comment, CommentCreatePayload>>,
    pub edit: GlobalSignal<HashMap<i32, StateFrame<(), CommentUpdatePayload>>>,
    pub remove: GlobalSignal<HashMap<i32, StateFrame>>,
    pub flags: GlobalSignal<StateFrame<Vec<CommentFlag>>>,
    pub flag_actions: GlobalSignal<HashMap<i32, StateFrame<(), CommentFlagPayload>>>,
    pub moderation: GlobalSignal<HashMap<i32, StateFrame>>,
    pub summaries: GlobalSignal<HashMap<i32, StateFrame<CommentFlagSummary>>>,
}

impl PartialEq for CommentState {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl CommentState {
    pub fn new() -> Self {
        Self {
            list: GlobalSignal::new(|| StateFrame::new()),
            view: GlobalSignal::new(|| HashMap::new()),
            add: GlobalSignal::new(|| StateFrame::new()),
            edit: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            flags: GlobalSignal::new(|| StateFrame::new()),
            flag_actions: GlobalSignal::new(|| HashMap::new()),
            moderation: GlobalSignal::new(|| HashMap::new()),
            summaries: GlobalSignal::new(|| HashMap::new()),
        }
    }

    pub fn reset(&self) {
        *self.list.write() = StateFrame::new();
        *self.view.write() = HashMap::new();
        *self.add.write() = StateFrame::new();
        *self.edit.write() = HashMap::new();
        *self.remove.write() = HashMap::new();
        *self.flags.write() = StateFrame::new();
        *self.flag_actions.write() = HashMap::new();
        *self.moderation.write() = HashMap::new();
        *self.summaries.write() = HashMap::new();
    }
}

static COMMENT_STATE: OnceLock<CommentState> = OnceLock::new();

pub fn use_comments() -> &'static CommentState {
    COMMENT_STATE.get_or_init(CommentState::new)
}
