use crate::store::{ListQuery, ListStore, Media, PaginatedList, StateFrame};
use crate::types::SortParam;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub color: String,
    pub text_color: String,
    pub is_active: bool,
    pub cover: Option<Media>,
    pub logo: Option<Media>,
    pub cover_id: Option<i32>,
    pub description: Option<String>,
    pub logo_id: Option<i32>,
    pub parent_id: Option<i32>,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            slug: String::new(),
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_else(|| Utc::now()),
            updated_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_else(|| Utc::now()),
            color: "#3b82f6".to_string(),
            text_color: "#ffffff".to_string(),
            is_active: true,
            cover: None,
            logo: None,
            cover_id: None,
            description: None,
            logo_id: None,
            parent_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CategoriesListQuery {
    pub page: u64,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub is_active: Option<bool>,
    pub parent_id: Option<i32>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
}

impl CategoriesListQuery {
    pub fn new() -> Self {
        Self {
            page: 1,
            ..Default::default()
        }
    }
}

impl ListQuery for CategoriesListQuery {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CategoriesAddPayload {
    pub name: String,
    pub slug: String,
    pub color: String,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
    pub cover_id: Option<i32>,
    pub description: Option<String>,
    pub logo_id: Option<i32>,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CategoriesEditPayload {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub parent_id: Option<Option<i32>>,
    pub description: Option<Option<String>>,
    pub cover_id: Option<Option<i32>>,
    pub logo_id: Option<Option<i32>>,
    pub color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

pub struct CategoriesState {
    pub add: GlobalSignal<StateFrame<(), CategoriesAddPayload>>,
    pub edit: GlobalSignal<HashMap<i32, StateFrame<(), CategoriesEditPayload>>>,
    pub remove: GlobalSignal<HashMap<i32, StateFrame>>,
    pub list: GlobalSignal<StateFrame<PaginatedList<Category>>>,
    pub view: GlobalSignal<HashMap<i32, StateFrame<Category>>>,
}

impl CategoriesState {
    pub fn new() -> Self {
        Self {
            add: GlobalSignal::new(|| StateFrame::new()),
            edit: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            list: GlobalSignal::new(|| StateFrame::new()),
            view: GlobalSignal::new(|| HashMap::new()),
        }
    }
}

static CATEGORIES_STATE: std::sync::OnceLock<CategoriesState> = std::sync::OnceLock::new();

impl ListStore<Category, CategoriesListQuery> for CategoriesState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<Category>>> {
        &self.list
    }

    async fn fetch_list(&self) {
        self.list().await;
    }

    async fn fetch_list_with_query(&self, query: CategoriesListQuery) {
        self.list_with_query(query).await;
    }
}

pub fn use_categories() -> &'static CategoriesState {
    CATEGORIES_STATE.get_or_init(|| CategoriesState::new())
}
