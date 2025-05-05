use serde::{Deserialize, Serialize};
use validator::Validate;
use axum::body::Bytes;

use crate::db::sea_models::asset::{AssetQuery, NewAsset, UpdateAsset};

// Struct for file upload with validation
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UploadAssetPayload {
    // These will be extracted from the multipart form data
    // and populated in the controller
    #[serde(skip)]
    pub file_data: Option<Bytes>,
    
    #[serde(skip)]
    pub file_name: Option<String>,
    
    #[serde(skip)]
    pub mime_type: Option<String>,
    
    #[serde(skip)]
    pub file_size: Option<i32>,
    
    // Optional context field to categorize assets
    pub context: Option<String>,
    
    // Optional owner_id field if not derived from auth
    pub owner_id: Option<i32>,
}

impl V1UploadAssetPayload {
    pub fn into_new_asset(self, file_url: String) -> NewAsset {
        NewAsset {
            file_url,
            file_name: self.file_name,
            mime_type: self.mime_type,
            size: self.file_size,
            owner_id: self.owner_id,
            context: self.context,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateAssetPayload {
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i32>,
    pub owner_id: Option<i32>,
    pub context: Option<String>,
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
    pub context: Option<String>,
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