use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

/// Payload to create a new post revision
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePostRevision {
    pub post_id: i32,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Query parameters for listing revisions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostRevisionListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u64>,
}

/// Lightweight projection for listing revisions
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct PostRevisionSummary {
    pub id: i32,
    pub post_id: i32,
    pub created_at: DateTimeWithTimeZone,
}

/// Payload to restore a specific revision into a post
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestorePostRevision {
    pub post_id: i32,
    pub revision_id: i32,
}
