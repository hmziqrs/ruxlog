use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::tag::{NewTag, TagQuery, UpdateTag};
use crate::utils::SortParam;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreateTagPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub color: Option<String>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

impl V1CreateTagPayload {
    pub fn into_new_tag(self) -> NewTag {
        NewTag {
            name: self.name,
            slug: self.slug,
            description: self.description,
            color: self.color,
            text_color: self.text_color,
            is_active: self.is_active,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateTagPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub slug: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub color: Option<String>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

impl V1UpdateTagPayload {
    pub fn into_update_tag(self) -> UpdateTag {
        UpdateTag {
            name: self.name,
            slug: self.slug,
            description: self.description,
            color: self.color,
            text_color: self.text_color,
            is_active: self.is_active,
            updated_at: chrono::Utc::now().fixed_offset(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1TagQueryParams {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>, // [{ field, order }]
    pub is_active: Option<bool>,
}

impl V1TagQueryParams {
    pub fn into_query(self) -> TagQuery {
        TagQuery {
            page: self.page,
            search: self.search,
            sorts: self.sorts,
            is_active: self.is_active,
        }
    }
}
