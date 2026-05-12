use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ledger_entry_type")]
#[serde(rename_all = "snake_case")]
pub enum LedgerEntryType {
    #[sea_orm(string_value = "credit")]
    Credit,
    #[sea_orm(string_value = "debit")]
    Debit,
    #[sea_orm(string_value = "payout")]
    Payout,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payout_ledgers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub amount_cents: i32,
    pub currency: String,
    pub entry_type: LedgerEntryType,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub description: Option<String>,
    pub balance_after: i32,
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
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
