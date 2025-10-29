use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::asset::{AssetContext, AssetQuery, UpdateAsset};
use crate::utils::SortParam;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateAssetPayload {
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i32>,
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
}

impl V1UpdateAssetPayload {
    pub fn into_update_asset(self) -> UpdateAsset {
        UpdateAsset {
            file_url: self.file_url,
            file_name: self.file_name,
            mime_type: self.mime_type,
            size: self.size,
            owner_id: self.owner_id,
            context: self.context,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1AssetQueryParams {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
    // Date range filters
    pub uploaded_at_gt: Option<DateTimeWithTimeZone>,
    pub uploaded_at_lt: Option<DateTimeWithTimeZone>,
}

impl V1AssetQueryParams {
    pub fn into_asset_query(self) -> AssetQuery {
        AssetQuery {
            page_no: self.page,
            search: self.search,
            sorts: self.sorts,
            owner_id: self.owner_id,
            context: self.context,
            uploaded_at_gt: self.uploaded_at_gt,
            uploaded_at_lt: self.uploaded_at_lt,
        }
    }
}
