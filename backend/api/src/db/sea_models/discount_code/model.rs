use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "discount_type")]
#[serde(rename_all = "snake_case")]
pub enum DiscountType {
    #[sea_orm(string_value = "percentage")]
    Percentage,
    #[sea_orm(string_value = "fixed_amount")]
    FixedAmount,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "discount_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: DiscountType,
    pub discount_value: i32,
    pub currency: Option<String>,
    pub max_redemptions: Option<i32>,
    pub redeemed_count: i32,
    pub valid_from: Option<DateTimeWithTimeZone>,
    pub valid_until: Option<DateTimeWithTimeZone>,
    pub plan_id: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::plan::Entity",
        from = "Column::PlanId",
        to = "super::super::plan::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Plan,
}

impl Related<super::super::plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plan.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
