use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_revisions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub post_id: i32,

    pub content: String,

    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub metadata: Option<serde_json::Value>,

    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::post::Entity",
        from = "Column::PostId",
        to = "super::super::post::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Post,
}

impl Related<super::super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
