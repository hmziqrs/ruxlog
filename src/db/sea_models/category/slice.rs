use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewCategory {
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub logo_image: Option<String>,
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
    pub cover_image: Option<Option<String>>,
    pub logo_image: Option<Option<String>>,
    pub color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CategoryQuery {
    pub page_no: Option<u64>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
    pub parent_id: Option<i32>,
}
