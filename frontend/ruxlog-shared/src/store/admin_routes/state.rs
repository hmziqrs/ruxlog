use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::StateFrame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteStatus {
    pub id: i32,
    pub pattern: String,
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
    pub is_blocked: Option<bool>,
    pub reason: Option<String>,
}

pub struct AdminRoutesState {
    pub list: GlobalSignal<StateFrame<Vec<RouteStatus>>>,
    pub block: GlobalSignal<StateFrame<RouteStatus, BlockRoutePayload>>,
    pub update: GlobalSignal<HashMap<String, StateFrame<(), UpdateRoutePayload>>>,
    pub remove: GlobalSignal<HashMap<String, StateFrame>>,
    pub sync: GlobalSignal<StateFrame>,
}

impl AdminRoutesState {
    pub fn new() -> Self {
        Self {
            list: GlobalSignal::new(|| StateFrame::new()),
            block: GlobalSignal::new(|| StateFrame::new()),
            update: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            sync: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub fn reset(&self) {
        *self.list.write() = StateFrame::new();
        *self.block.write() = StateFrame::new();
        *self.update.write() = HashMap::new();
        *self.remove.write() = HashMap::new();
        *self.sync.write() = StateFrame::new();
    }
}

static ADMIN_ROUTES_STATE: OnceLock<AdminRoutesState> = OnceLock::new();

pub fn use_admin_routes() -> &'static AdminRoutesState {
    ADMIN_ROUTES_STATE.get_or_init(AdminRoutesState::new)
}
