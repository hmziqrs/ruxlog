use super::model::AssetContext;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewAsset {
    pub file_url: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i32>,
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateAsset {
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i32>,
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AssetQuery {
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
    pub search: Option<String>,
    pub page_no: Option<u64>,
    pub sorts: Option<Vec<crate::utils::SortParam>>,
    // Date range filters
    pub uploaded_at_gt: Option<DateTimeWithTimeZone>,
    pub uploaded_at_lt: Option<DateTimeWithTimeZone>,
}
