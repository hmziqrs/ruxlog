use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub use ruxlog_types::enums::UserRole;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub avatar_id: Option<i32>,
    pub is_verified: bool,
    pub role: UserRole,
    pub two_fa_enabled: bool,
    pub two_fa_secret: Option<String>,
    pub two_fa_backup_codes: Option<Json>,
    pub google_id: Option<String>,
    pub oauth_provider: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl Model {
    pub fn get_role(&self) -> UserRole {
        self.role
    }

    pub fn is_user(&self) -> bool {
        self.get_role().to_i32() >= UserRole::User.to_i32()
    }

    pub fn is_author(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Author.to_i32()
    }

    pub fn is_moderator(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Moderator.to_i32()
    }

    pub fn is_admin(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Admin.to_i32()
    }

    pub fn is_super_admin(&self) -> bool {
        self.get_role().to_i32() >= UserRole::SuperAdmin.to_i32()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::super::email_verification::Entity")]
    EmailVerification,
    #[sea_orm(has_many = "super::super::forgot_password::Entity")]
    ForgotPassword,
    #[sea_orm(has_many = "super::super::post::Entity")]
    Post,
    #[sea_orm(
        belongs_to = "super::super::media::Entity",
        from = "Column::AvatarId",
        to = "super::super::media::Column::Id"
    )]
    Media,
}

impl Related<super::super::email_verification::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailVerification.def()
    }
}

impl Related<super::super::forgot_password::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ForgotPassword.def()
    }
}

impl Related<super::super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl Related<super::super::media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    // Add custom ActiveModel behavior here if needed
}
