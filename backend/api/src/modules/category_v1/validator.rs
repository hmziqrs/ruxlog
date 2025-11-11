use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    db::sea_models::category::{CategoryQuery, NewCategory, UpdateCategory},
    utils::SortParam,
};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreateCategoryPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    pub parent_id: Option<i32>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub cover_id: Option<i32>,
    pub logo_id: Option<i32>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub color: String,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

impl V1CreateCategoryPayload {
    pub fn into_new_category(self) -> NewCategory {
        NewCategory {
            name: self.name,
            slug: self.slug,
            parent_id: self.parent_id,
            description: self.description,
            cover_id: self.cover_id,
            logo_id: self.logo_id,
            color: Some(self.color),
            text_color: self.text_color,
            is_active: self.is_active,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateCategoryPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub slug: Option<String>,
    pub parent_id: Option<Option<i32>>,
    #[validate(length(max = 1000))]
    pub description: Option<Option<String>>,
    pub cover_id: Option<Option<i32>>,
    pub logo_id: Option<Option<i32>>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub color: Option<String>,
    #[validate(custom(function = "validate_hex_color"), skip)]
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

impl V1UpdateCategoryPayload {
    pub fn into_update_category(self) -> UpdateCategory {
        UpdateCategory {
            name: self.name,
            slug: self.slug,
            parent_id: self.parent_id,
            description: self.description,
            cover_id: self.cover_id,
            logo_id: self.logo_id,
            color: self.color,
            text_color: self.text_color,
            is_active: self.is_active,
            updated_at: chrono::Utc::now().fixed_offset(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CategoryQueryParams {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub parent_id: Option<i32>,
    pub is_active: Option<bool>,
    pub created_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at_lt: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub updated_at_lt: Option<chrono::DateTime<chrono::FixedOffset>>,
}

impl V1CategoryQueryParams {
    pub fn into_category_query(self) -> CategoryQuery {
        CategoryQuery {
            page: self.page,
            search: self.search,
            sorts: self.sorts,
            parent_id: self.parent_id,
            is_active: self.is_active,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
        }
    }
}
