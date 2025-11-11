use chrono::{DateTime, FixedOffset};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use crate::utils::SortParam;

#[derive(Deserialize, Debug)]
pub struct NewCategory {
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_id: Option<i32>,
    pub logo_id: Option<i32>,
    pub color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub parent_id: Option<Option<i32>>,
    pub description: Option<Option<String>>,
    pub cover_id: Option<Option<i32>>,
    pub logo_id: Option<Option<i32>>,
    pub color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CategoryQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub parent_id: Option<i32>,
    pub is_active: Option<bool>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryMedia {
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryWithRelations {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub color: String,
    pub text_color: String,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<CategoryMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<CategoryMedia>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct CategoryWithJoinedData {
    // Category fields
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_id: Option<i32>,
    pub logo_id: Option<i32>,
    pub color: String,
    pub text_color: String,
    pub is_active: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,

    // Cover media fields from join
    pub cover_object_key: Option<String>,
    pub cover_file_url: Option<String>,
    pub cover_mime_type: Option<String>,
    pub cover_width: Option<i32>,
    pub cover_height: Option<i32>,
    pub cover_size: Option<i64>,

    // Logo media fields from join
    pub logo_object_key: Option<String>,
    pub logo_file_url: Option<String>,
    pub logo_mime_type: Option<String>,
    pub logo_width: Option<i32>,
    pub logo_height: Option<i32>,
    pub logo_size: Option<i64>,
}

impl CategoryWithJoinedData {
    pub fn into_relation(self) -> CategoryWithRelations {
        let cover = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.cover_id,
            self.cover_object_key,
            self.cover_file_url,
            self.cover_mime_type,
            self.cover_size,
        ) {
            Some(CategoryMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.cover_width,
                height: self.cover_height,
                size,
            })
        } else {
            None
        };

        let logo = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.logo_id,
            self.logo_object_key,
            self.logo_file_url,
            self.logo_mime_type,
            self.logo_size,
        ) {
            Some(CategoryMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.logo_width,
                height: self.logo_height,
                size,
            })
        } else {
            None
        };

        CategoryWithRelations {
            id: self.id,
            name: self.name,
            slug: self.slug,
            parent_id: self.parent_id,
            description: self.description,
            color: self.color,
            text_color: self.text_color,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
            cover,
            logo,
        }
    }
}
