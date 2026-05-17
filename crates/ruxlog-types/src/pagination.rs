use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedList<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

impl<T> PaginatedList<T> {
    pub fn new(data: Vec<T>, total: u64, page: u64, per_page: u64) -> Self {
        Self {
            data,
            total,
            page,
            per_page,
        }
    }

    pub fn has_next_page(&self) -> bool {
        let total_pages = (self.total as f64 / self.per_page as f64).ceil() as u64;
        self.page < total_pages
    }

    pub fn has_previous_page(&self) -> bool {
        self.page > 1
    }
}

impl<T> Deref for PaginatedList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for PaginatedList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> IntoIterator for PaginatedList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
