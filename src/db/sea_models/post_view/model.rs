use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_views")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub post_id: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::post::Entity",
        from = "Column::PostId",
        to = "super::super::post::Column::Id"
    )]
    Post,
}

impl Related<super::super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}