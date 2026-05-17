use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub use ruxlog_types::enums::MediaReference;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "media")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub bucket: Option<String>,
    pub object_key: String,
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
