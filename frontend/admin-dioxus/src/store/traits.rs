use crate::store::{PaginatedList, StateFrame};
use crate::types::SortParam;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub trait ListQuery: Clone + Default + Serialize + for<'de> Deserialize<'de> + PartialEq {
    fn new() -> Self;

    fn page(&self) -> u64;

    fn set_page(&mut self, page: u64);

    fn search(&self) -> Option<String>;

    fn set_search(&mut self, search: Option<String>);

    fn sorts(&self) -> Option<Vec<SortParam>>;

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>);
}

pub trait ListStore<T, Q>
where
    T: Clone + PartialEq + 'static,
    Q: ListQuery,
{
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<T>>>;

    fn fetch_list(&self) -> impl std::future::Future<Output = ()>;

    fn fetch_list_with_query(&self, query: Q) -> impl std::future::Future<Output = ()>;
}

/// Base structure for common list query fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseListQuery {
    pub page: u64,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
}

impl Default for BaseListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            search: None,
            sorts: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        }
    }
}

impl BaseListQuery {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ListQuery for BaseListQuery {
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
