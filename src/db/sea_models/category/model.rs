use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::sea_models::media;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_id: Option<i32>,
    pub logo_id: Option<i32>,
    pub color: String,
    pub text_color: String,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "media::Entity",
        from = "Column::CoverId",
        to = "media::Column::Id"
    )]
    Cover,
    #[sea_orm(
        belongs_to = "media::Entity",
        from = "Column::LogoId",
        to = "media::Column::Id"
    )]
    Logo,
}

impl ActiveModelBehavior for ActiveModel {}
