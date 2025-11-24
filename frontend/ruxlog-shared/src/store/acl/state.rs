use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{ListQuery, ListStore, PaginatedList, StateFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConstant {
    pub key: String,
    pub value: String,
    pub value_type: Option<String>,
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub source: String,
    pub updated_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AclListQuery {
    pub page: u64,
    pub per_page: u64,
    pub search: Option<String>,
    pub is_sensitive: Option<bool>,
    pub value_type: Option<String>,
}

impl AclListQuery {
    pub fn new() -> Self {
        Self {
            page: 1,
            per_page: 20,
            ..Default::default()
        }
    }
}

impl ListQuery for AclListQuery {
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

    fn sorts(&self) -> Option<Vec<oxstore::SortParam>> {
        None
    }

    fn set_sorts(&mut self, sorts: Option<Vec<oxstore::SortParam>>) {
        let _ = sorts;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AclCreatePayload {
    pub key: String,
    pub value: String,
    pub value_type: Option<String>,
    pub description: Option<String>,
    pub is_sensitive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AclUpdatePayload {
    pub value: Option<String>,
    pub value_type: Option<String>,
    pub description: Option<String>,
    pub is_sensitive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AclSyncResponse {
    pub message: Option<String>,
}

pub struct AclState {
    pub list: GlobalSignal<StateFrame<PaginatedList<AppConstant>>>,
    pub create: GlobalSignal<StateFrame<AppConstant, AclCreatePayload>>,
    pub update: GlobalSignal<HashMap<String, StateFrame<(), AclCreatePayload>>>,
    pub remove: GlobalSignal<HashMap<String, StateFrame>>,
    pub sync: GlobalSignal<StateFrame<AclSyncResponse>>,
}

impl AclState {
    pub fn new() -> Self {
        Self {
            list: GlobalSignal::new(|| StateFrame::new()),
            create: GlobalSignal::new(|| StateFrame::new()),
            update: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            sync: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub fn reset(&self) {
        *self.list.write() = StateFrame::new();
        *self.create.write() = StateFrame::new();
        *self.update.write() = HashMap::new();
        *self.remove.write() = HashMap::new();
        *self.sync.write() = StateFrame::new();
    }
}

static ACL_STATE: std::sync::OnceLock<AclState> = std::sync::OnceLock::new();

pub fn use_acl() -> &'static AclState {
    ACL_STATE.get_or_init(AclState::new)
}

impl ListStore<AppConstant, AclListQuery> for AclState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<AppConstant>>> {
        &self.list
    }

    async fn fetch_list(&self) {
        self.list().await;
    }

    async fn fetch_list_with_query(&self, query: AclListQuery) {
        self.list_with_query(query).await;
    }
}
