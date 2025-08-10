use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "scheduled_post_status"
)]
pub enum ScheduledPostStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "published")]
    Published,
    #[sea_orm(string_value = "canceled")]
    Canceled,
    #[sea_orm(string_value = "failed")]
    Failed,
}

impl fmt::Display for ScheduledPostStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Published => "published",
            Self::Canceled => "canceled",
            Self::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

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
