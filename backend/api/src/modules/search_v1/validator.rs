//! Search request/response types.

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct SearchQuery {
    #[validate(length(min = 1, max = 200))]
    pub q: String,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub per_page: Option<u64>,
}

impl SearchQuery {
    pub fn page(&self) -> u64 {
        // DOS-SEARCH-1: cap the page so a caller cannot drive an arbitrarily
        // large (and therefore expensive) OFFSET on top of the unindexed
        // triple-ILIKE scan. 500 pages × 100 per_page bounds the offset to ~50k.
        self.page.unwrap_or(1).clamp(1, 500)
    }

    pub fn per_page(&self) -> u64 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> u64 {
        (self.page().saturating_sub(1)) * self.per_page()
    }
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub status: String,
    pub published_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub rank: f64,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub data: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Debug, Serialize)]
pub struct SearchMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub query: String,
}
