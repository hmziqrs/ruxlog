use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::SubscriberStatus;
use crate::utils::SortParam;

/// New subscriber DTO for insertion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSubscriber {
    pub email: String,
    pub status: SubscriberStatus,
    pub token: String,
}

/// Update subscriber DTO for partial updates (e.g., confirm/unsubscribe)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSubscriber {
    pub status: Option<SubscriberStatus>,
    pub token: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
}

/// Query parameters for searching/paginating subscribers
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SubscriberQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub status: Option<SubscriberStatus>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

/// Lightweight subscriber list item for admin listings
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct SubscriberListItem {
    pub id: i32,
    pub email: String,
    pub status: SubscriberStatus,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
