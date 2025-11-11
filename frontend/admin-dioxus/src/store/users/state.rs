use crate::store::auth::UserRole;
use crate::store::media::Media;
use crate::store::traits::{ListQuery, ListStore};
use crate::store::{PaginatedList, StateFrame};
use crate::types::SortParam;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub avatar: Option<Media>,
    pub created_at: DateTime<Utc>,
    pub email: String,
    pub id: i32,
    pub is_verified: bool,
    pub name: String,
    pub role: UserRole,
    pub two_fa_backup_codes: Option<String>,
    pub two_fa_enabled: bool,
    pub two_fa_secret: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsersAddPayload {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub avatar_id: Option<i32>,
    #[serde(default)]
    pub is_verified: bool,
}

impl Default for UsersAddPayload {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            password: String::new(),
            role: "user".to_string(),
            avatar_id: None,
            is_verified: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UsersEditPayload {
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_id: Option<i32>,
    pub password: Option<String>,
    pub is_verified: Option<bool>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsersListQuery {
    pub page: Option<u64>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub status: Option<bool>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
}

impl Default for UsersListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            email: None,
            name: None,
            role: None,
            status: None,
            sorts: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        }
    }
}

impl UsersListQuery {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ListQuery for UsersListQuery {
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
        // Combine email and name for search
        if self.email.is_some() || self.name.is_some() {
            Some(format!(
                "{}{}",
                self.email.as_deref().unwrap_or(""),
                self.name.as_deref().unwrap_or("")
            ))
        } else {
            None
        }
    }

    fn set_search(&mut self, search: Option<String>) {
        // Set both email and name to the search term
        self.email = search.clone();
        self.name = search;
    }

    fn sorts(&self) -> Option<Vec<SortParam>> {
        self.sorts.clone()
    }

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>) {
        self.sorts = sorts;
    }
}

pub struct UsersState {
    pub add: GlobalSignal<StateFrame<(), UsersAddPayload>>,
    pub edit: GlobalSignal<HashMap<i32, StateFrame<(), UsersEditPayload>>>,
    pub remove: GlobalSignal<HashMap<i32, StateFrame>>,
    pub list: GlobalSignal<StateFrame<PaginatedList<User>>>,
    pub view: GlobalSignal<HashMap<i32, StateFrame<User>>>,
}

impl UsersState {
    pub fn new() -> Self {
        UsersState {
            add: GlobalSignal::new(|| StateFrame::new()),
            edit: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            list: GlobalSignal::new(|| StateFrame::new()),
            view: GlobalSignal::new(|| HashMap::new()),
        }
    }
}

static USER_STATE: std::sync::OnceLock<UsersState> = std::sync::OnceLock::new();

pub fn use_user() -> &'static UsersState {
    USER_STATE.get_or_init(|| UsersState::new())
}

impl ListStore<User, UsersListQuery> for UsersState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<User>>> {
        &self.list
    }

    async fn fetch_list(&self) {
        self.list().await;
    }

    async fn fetch_list_with_query(&self, query: UsersListQuery) {
        self.list_with_query(query).await;
    }
}
