use serde::{Deserialize, Serialize};
use validator::Validate;

fn validate_hex_color(s: &str) -> Result<(), validator::ValidationError> {
    let s = s.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    let ok = s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit());
    if ok {
        Ok(())
    } else {
        Err(validator::ValidationError::new("hex_color"))
    }
}

use crate::db::sea_models::tag::{NewTag, TagQuery, UpdateTag};

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
    pub sort_order: Option<String>,
}

impl V1TagQueryParams {
    pub fn into_query(self) -> TagQuery {
        TagQuery {
            page: self.page,
            search: self.search,
            sort_order: self.sort_order,
        }
    }
}
