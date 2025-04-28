use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Define the user role enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
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
}

// Define the entity for 'users' table
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub avatar: Option<String>,
    pub is_verified: bool,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Define the relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::super::email_verification::Entity")]
    EmailVerification,
    #[sea_orm(has_many = "super::super::forgot_password::Entity")]
    ForgotPassword,
    #[sea_orm(has_many = "super::super::post::Entity")]
    Post,
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

// ActiveModel is the mutable version of Model
impl ActiveModelBehavior for ActiveModel {
    // Add custom ActiveModel behavior here if needed
}
