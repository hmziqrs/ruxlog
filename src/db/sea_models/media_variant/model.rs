use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::media;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "media_variants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub media_id: i32,
    pub object_key: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
    pub extension: Option<String>,
    pub quality: Option<i32>,
    pub variant_type: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "media::Entity",
        from = "Column::MediaId",
        to = "media::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Media,
}

impl Related<media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
