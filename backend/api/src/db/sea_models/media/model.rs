use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "media_reference_type"
)]
#[serde(rename_all = "kebab-case")]
pub enum MediaReference {
    #[sea_orm(string_value = "category")]
    Category,
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "post")]
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

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "media")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
    pub extension: Option<String>,
    pub uploader_id: Option<i32>,
    pub reference_type: Option<MediaReference>,
    pub content_hash: Option<String>,
    pub is_optimized: bool,
    pub optimized_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UploaderId",
        to = "super::super::user::Column::Id"
    )]
    Uploader,
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Uploader.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn pixel_dimensions(&self) -> Option<(i32, i32)> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }

    pub fn with_usage(&self, usage_count: i64) -> super::slice::MediaWithUsage {
        super::slice::MediaWithUsage {
            media: self.clone(),
            usage_count,
        }
    }
}
