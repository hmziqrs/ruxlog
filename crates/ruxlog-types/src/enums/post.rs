use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "post_status")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PostStatus {
    #[serde(rename = "Draft")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "draft"))]
    #[default]
    Draft,
    #[serde(rename = "Published")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "published"))]
    Published,
    #[serde(rename = "Archived")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "archived"))]
    Archived,
}

impl fmt::Display for PostStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Published => write!(f, "published"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PostSortBy {
    Title,
    UpdatedAt,
    PublishedAt,
    ViewCount,
    LikesCount,
}
