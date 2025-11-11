use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

/// Payload to create a new post series
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPostSeries {
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Payload to update an existing post series
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePostSeries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Query parameters for listing series with pagination and optional search
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostSeriesListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

/// Projection for listing series with post counts
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct PostSeriesWithCount {
    pub id: i32,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub posts_count: i64,
}
