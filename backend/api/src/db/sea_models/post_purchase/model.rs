use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A user's one-time purchase of a gated post, granting permanent read access.
/// Created by the verified billing webhook from a server-bound checkout intent
/// (see `services/paywall` and plan Phase 4). Consulted by the paywall to decide
/// access to `PostAccessType::Paid` posts.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_purchases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    /// Optional link to the recorded payment row.
    pub payment_id: Option<i32>,
    /// Billing provider that processed the purchase (e.g. "stripe").
    pub provider: String,
    pub amount_cents: i32,
    pub currency: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UserId",
        to = "super::super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::super::post::Entity",
        from = "Column::PostId",
        to = "super::super::post::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Post,
    #[sea_orm(
        belongs_to = "super::super::payment::Entity",
        from = "Column::PaymentId",
        to = "super::super::payment::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Payment,
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl Related<super::super::payment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
