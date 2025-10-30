use crate::utils::SortParam;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use super::MediaReference;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewMedia {
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
    pub extension: Option<String>,
    pub uploader_id: Option<i32>,
    pub reference_type: Option<MediaReference>,
    pub content_hash: Option<String>,
    pub is_optimized: bool,
    pub optimized_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MediaDeletion {
    pub id: i32,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MediaQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>, // [{ field, order }]
    pub reference_type: Option<MediaReference>,
    pub uploader_id: Option<i32>,
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}
