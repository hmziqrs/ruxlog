use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::asset::{AssetContext, AssetQuery, UpdateAsset};

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
    pub sort_order: Option<String>,
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
}

impl V1AssetQueryParams {
    pub fn into_asset_query(self) -> AssetQuery {
        AssetQuery {
            page_no: self.page,
            search: self.search,
            sort_order: self.sort_order,
            owner_id: self.owner_id,
            context: self.context,
        }
    }
}
