use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "invoice_status")]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "sent")]
    Sent,
    #[sea_orm(string_value = "paid")]
    Paid,
    #[sea_orm(string_value = "void")]
    Void,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "invoices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub subscription_id: Option<i32>,
    pub payment_id: Option<i32>,
    pub invoice_number: String,
    pub amount_cents: i32,
    pub currency: String,
    pub status: InvoiceStatus,
    pub due_date: Option<DateTimeWithTimeZone>,
    pub paid_at: Option<DateTimeWithTimeZone>,
    pub pdf_url: Option<String>,
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

impl Related<super::super::subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::super::payment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
