use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{ListQuery, ListStore, PaginatedList, SortParam, StateFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockFilter {
    All,
    Blocked,
    Unblocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteStatus {
    pub id: i32,
    #[serde(rename = "route_pattern")]
    pub route_pattern: String,
    pub is_blocked: bool,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockRoutePayload {
    pub pattern: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateRoutePayload {
    pub is_blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AdminRoutesListQuery {
    pub page: Option<u64>,
    pub block_filter: Option<BlockFilter>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
}

impl AdminRoutesListQuery {
    pub fn new() -> Self {
        Self {
            page: Some(1),
            ..Default::default()
        }
    }
}

impl ListQuery for AdminRoutesListQuery {
    fn new() -> Self {
        Self::new()
    }

    fn page(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    fn set_page(&mut self, page: u64) {
        self.page = Some(page);
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RouteSyncIntervalStatus {
    pub interval_secs: u64,
    #[serde(default)]
    pub paused: bool,
    #[serde(default)]
    pub is_running: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub remaining_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateSyncIntervalPayload {
    pub interval_secs: u64,
}

pub struct AdminRoutesState {
    pub list: GlobalSignal<StateFrame<PaginatedList<RouteStatus>>>,
    pub block: GlobalSignal<StateFrame<RouteStatus, BlockRoutePayload>>,
    pub update: GlobalSignal<HashMap<String, StateFrame<(), UpdateRoutePayload>>>,
    pub remove: GlobalSignal<HashMap<String, StateFrame>>,
    pub sync: GlobalSignal<StateFrame>,
    pub sync_interval: GlobalSignal<StateFrame<RouteSyncIntervalStatus, UpdateSyncIntervalPayload>>,
}

impl PartialEq for AdminRoutesState {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl AdminRoutesState {
    pub fn new() -> Self {
        Self {
            list: GlobalSignal::new(|| StateFrame::new()),
            block: GlobalSignal::new(|| StateFrame::new()),
            update: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            sync: GlobalSignal::new(|| StateFrame::new()),
            sync_interval: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub fn reset(&self) {
        *self.list.write() = StateFrame::new();
        *self.block.write() = StateFrame::new();
        *self.update.write() = HashMap::new();
        *self.remove.write() = HashMap::new();
        *self.sync.write() = StateFrame::new();
        *self.sync_interval.write() = StateFrame::new();
    }
}

static ADMIN_ROUTES_STATE: OnceLock<AdminRoutesState> = OnceLock::new();

pub fn use_admin_routes() -> &'static AdminRoutesState {
    ADMIN_ROUTES_STATE.get_or_init(AdminRoutesState::new)
}

impl ListStore<RouteStatus, AdminRoutesListQuery> for AdminRoutesState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<RouteStatus>>> {
        &self.list
    }

    async fn fetch_list(&self) {
        self.list().await;
    }

    async fn fetch_list_with_query(&self, query: AdminRoutesListQuery) {
        self.list_with_query(query).await;
    }
}
