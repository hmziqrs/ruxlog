use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub use ruxlog_types::enums::ScheduledPostStatus;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "scheduled_posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub post_id: i32,

    pub publish_at: DateTimeWithTimeZone,

    pub status: ScheduledPostStatus,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
