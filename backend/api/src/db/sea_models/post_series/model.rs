use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_series")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub name: String,
    pub slug: String,

    #[sea_orm(nullable)]
    pub description: Option<String>,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::super::post_series_post::Entity")]
    SeriesPost,
}

impl Related<super::super::post_series_post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SeriesPost.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
