use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seed_runs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub key: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub ranges: serde_json::Value,
    pub ran_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

