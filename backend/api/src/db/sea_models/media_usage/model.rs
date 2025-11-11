use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::media;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "entity_type")]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    #[sea_orm(string_value = "category")]
    Category,
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "post")]
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

    pub fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "category" => Ok(EntityType::Category),
            "user" => Ok(EntityType::User),
            "post" => Ok(EntityType::Post),
            other => Err(format!("Invalid entity type: {}", other)),
        }
    }
}

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
