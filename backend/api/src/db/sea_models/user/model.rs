use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    #[sea_orm(string_value = "super-admin")]
    SuperAdmin,
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "moderator")]
    Moderator,
    #[sea_orm(string_value = "author")]
    Author,
    #[sea_orm(string_value = "user")]
    User,
}

impl UserRole {
    pub fn to_i32(&self) -> i32 {
        match self {
            UserRole::SuperAdmin => 4,
            UserRole::Admin => 3,
            UserRole::Moderator => 2,
            UserRole::Author => 1,
            UserRole::User => 0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            UserRole::SuperAdmin => "super-admin".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Moderator => "moderator".to_string(),
            UserRole::Author => "author".to_string(),
            UserRole::User => "user".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "super-admin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "author" => Ok(UserRole::Author),
            "user" => Ok(UserRole::User),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        UserRole::from_str(s).unwrap_or(UserRole::User)
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UserRole::from_str(s)
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        role.to_i32()
    }
}

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
