use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "plan_interval")]
#[serde(rename_all = "lowercase")]
pub enum PlanInterval {
    #[sea_orm(string_value = "monthly")]
    Monthly,
    #[sea_orm(string_value = "yearly")]
    Yearly,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub interval: PlanInterval,
    pub trial_days: i32,
    pub features: Option<Json>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
