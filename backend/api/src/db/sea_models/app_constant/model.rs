use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "app_constants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub key: String,

    pub value: String,

    #[sea_orm(nullable)]
    pub value_type: Option<String>,

    #[sea_orm(nullable)]
    pub description: Option<String>,

    pub is_sensitive: bool,

    pub source: String,

    #[sea_orm(nullable)]
    pub updated_by: Option<i32>,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().fixed_offset()),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
            source: Set("env".to_string()),
            ..ActiveModelTrait::default()
        }
    }
}
