use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::tag::{NewTag, TagQuery, UpdateTag};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreateTagPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

impl V1CreateTagPayload {
    pub fn into_new_tag(self) -> NewTag {
        NewTag {
            name: self.name,
            slug: self.slug,
            description: self.description,
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
}

impl V1UpdateTagPayload {
    pub fn into_update_tag(self) -> UpdateTag {
        UpdateTag {
            name: self.name,
            slug: self.slug,
            description: self.description,
            updated_at: chrono::Utc::now().fixed_offset(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1TagQueryParams {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
}

impl V1TagQueryParams {
    pub fn into_query(self) -> TagQuery {
        TagQuery {
            page_no: self.page,
            search: self.search,
            sort_order: self.sort_order,
        }
    }
}
