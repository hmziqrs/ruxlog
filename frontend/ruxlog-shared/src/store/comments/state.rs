use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{ListQuery, ListStore, PaginatedList, SortParam, StateFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentAuthor {
    pub id: i32,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HiddenFilter {
    All,
    Hidden,
    Visible,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlagFilter {
    All,
    Flagged,
    NotFlagged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub content: String,
    pub parent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    #[serde(default, alias = "is_hidden")]
    pub hidden: bool,
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
    pub page: u64,
    pub limit: Option<u64>,
    pub hidden_filter: Option<HiddenFilter>,
    pub flag_filter: Option<FlagFilter>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
}

impl CommentListQuery {
    pub fn new() -> Self {
        Self {
            page: 1,
            ..Default::default()
        }
    }
}

impl ListQuery for CommentListQuery {
    fn new() -> Self {
        Self::new()
    }

    fn page(&self) -> u64 {
        self.page
    }

    fn set_page(&mut self, page: u64) {
        self.page = page;
    }

    fn search(&self) -> Option<String> {
        self.search.clone()
    }

    fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    fn sorts(&self) -> Option<Vec<SortParam>> {
        self.sorts.clone()
    }

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>) {
        self.sorts = sorts;
    }
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

impl ListStore<Comment, CommentListQuery> for CommentState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<Comment>>> {
        &self.list
    }

    async fn fetch_list(&self) {
        self.admin_list(CommentListQuery::new()).await;
    }

    async fn fetch_list_with_query(&self, query: CommentListQuery) {
        self.admin_list(query).await;
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
