use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use crate::utils::SortParam;

#[derive(Deserialize, Debug)]
pub struct NewTag {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub text_color: Option<String>,
    pub is_active: Option<bool>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TagQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
    // New: dynamic multi-field sorts
    pub sorts: Option<Vec<SortParam>>,
    // Optional filter for active state
    pub is_active: Option<bool>,
}
