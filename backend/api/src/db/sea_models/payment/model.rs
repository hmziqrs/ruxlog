use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "payment_status")]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "refunded")]
    Refunded,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub subscription_id: Option<i32>,
    pub plan_id: Option<i32>,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub amount_cents: i32,
    pub currency: String,
    pub status: PaymentStatus,
    pub description: Option<String>,
    pub metadata: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
        belongs_to = "super::super::subscription::Entity",
        from = "Column::SubscriptionId",
        to = "super::super::subscription::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Subscription,
    #[sea_orm(
        belongs_to = "super::super::plan::Entity",
        from = "Column::PlanId",
        to = "super::super::plan::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Plan,
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::super::subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::super::plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plan.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
