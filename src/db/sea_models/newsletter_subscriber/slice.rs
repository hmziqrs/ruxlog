use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::SubscriberStatus;

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
}

/// Query parameters for searching/paginating subscribers
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SubscriberQuery {
    pub page_no: Option<u64>,
    pub search: Option<String>,
    pub status: Option<SubscriberStatus>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
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
