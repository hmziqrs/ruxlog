use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::media;

pub use ruxlog_types::enums::EntityType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "media_usage")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub media_id: i32,
    pub entity_type: EntityType,
    pub entity_id: i32,
    pub field_name: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "media::Entity",
        from = "Column::MediaId",
        to = "media::Column::Id"
    )]
    Media,
}

impl Related<media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
