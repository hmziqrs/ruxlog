use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::category::{CategoryQuery, NewCategory, UpdateCategory};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreateCategoryPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    pub parent_id: Option<i32>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub logo_image: Option<String>,
}

impl V1CreateCategoryPayload {
    pub fn into_new_category(self) -> NewCategory {
        NewCategory {
            name: self.name,
            slug: self.slug,
            parent_id: self.parent_id,
            description: self.description,
            cover_image: self.cover_image,
            logo_image: self.logo_image,
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
    pub cover_image: Option<Option<String>>,
    pub logo_image: Option<Option<String>>,
}

impl V1UpdateCategoryPayload {
    pub fn into_update_category(self) -> UpdateCategory {
        UpdateCategory {
            name: self.name,
            slug: self.slug,
            parent_id: self.parent_id,
            description: self.description,
            cover_image: self.cover_image,
            logo_image: self.logo_image,
            updated_at: chrono::Utc::now().fixed_offset(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CategoryQueryParams {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
    pub parent_id: Option<i32>,
}

impl V1CategoryQueryParams {
    pub fn into_category_query(self) -> CategoryQuery {
        CategoryQuery {
            page_no: self.page,
            search: self.search,
            sort_order: self.sort_order,
            parent_id: self.parent_id,
        }
    }
}
