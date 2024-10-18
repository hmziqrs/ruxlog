use chrono::NaiveDateTime;
use garde::{self, Validate};
use serde::{Deserialize, Serialize};

use crate::db::models::post::{Pagination, SortBy};
use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1CreatePostPayload {
    #[garde(length(min = 1, max = 255))]
    pub title: String,
    #[garde(length(min = 1))]
    pub content: String,
    pub published_at: Option<NaiveDateTime>,
    pub is_published: bool,
    #[garde(length(min = 1, max = 255))]
    pub slug: String,
    #[garde(length(max = 500))]
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub category_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1UpdatePostPayload {
    #[garde(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[garde(length(min = 1))]
    pub content: Option<String>,
    pub published_at: Option<Option<NaiveDateTime>>,
    pub is_published: Option<bool>,
    #[garde(length(min = 1, max = 255))]
    pub slug: Option<String>,
    #[garde(length(max = 500))]
    pub excerpt: Option<Option<String>>,
    pub featured_image_url: Option<Option<String>>,
    pub category_id: Option<Option<i32>>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1PostQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<String>,
}
