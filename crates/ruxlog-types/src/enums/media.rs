use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(
        rs_type = "String",
        db_type = "Enum",
        enum_name = "media_reference_type"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MediaReference {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "category"))]
    Category,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "user"))]
    User,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "post"))]
    Post,
}

impl MediaReference {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaReference::Category => "category",
            MediaReference::User => "user",
            MediaReference::Post => "post",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "category" => Ok(MediaReference::Category),
            "user" => Ok(MediaReference::User),
            "post" => Ok(MediaReference::Post),
            other => Err(format!("Invalid media reference type: {}", other)),
        }
    }
}

impl std::str::FromStr for MediaReference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MediaReference::from_str(s)
    }
}

impl fmt::Display for MediaReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaReference::Post => write!(f, "Post"),
            MediaReference::Category => write!(f, "Category"),
            MediaReference::User => write!(f, "User"),
        }
    }
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "entity_type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "category"))]
    Category,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "user"))]
    User,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "post"))]
    Post,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Category => "category",
            EntityType::User => "user",
            EntityType::Post => "post",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "category" => Ok(EntityType::Category),
            "user" => Ok(EntityType::User),
            "post" => Ok(EntityType::Post),
            other => Err(format!("Invalid entity type: {}", other)),
        }
    }
}
