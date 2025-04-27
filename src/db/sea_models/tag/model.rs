use chrono::NaiveDateTime;
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, QuerySelect, Set};
use serde::{Deserialize, Serialize};

// Define the entity for 'tags' table
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Define the relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

// ActiveModel is the mutable version of Model
impl ActiveModelBehavior for ActiveModel {}
