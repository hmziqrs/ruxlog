use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use super::model::ScheduledPostStatus;

/// Payload to create a new scheduled post entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateScheduledPost {
    pub post_id: i32,
    pub publish_at: DateTimeWithTimeZone,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ScheduledPostStatus>,
}

/// Payload to upsert (create or update) a scheduled post for a given post.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpsertScheduledPost {
    pub post_id: i32,
    pub publish_at: DateTimeWithTimeZone,
}

/// Query parameters for listing scheduled posts by status with pagination.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledPostStatusQuery {
    pub status: ScheduledPostStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u64>,
}

/// Query parameters to fetch pending schedules due before or at a specific time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledPostDueQuery {
    pub until: DateTimeWithTimeZone,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}
