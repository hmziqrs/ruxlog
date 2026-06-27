use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "super-admin"))]
    SuperAdmin,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "admin"))]
    Admin,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "moderator"))]
    Moderator,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "author"))]
    Author,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "user"))]
    #[default]
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

    #[allow(clippy::should_implement_trait)]
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

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "super-admin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Moderator => write!(f, "moderator"),
            UserRole::Author => write!(f, "author"),
            UserRole::User => write!(f, "user"),
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

// `Default` is derived on `UserRole` above (`#[default] User`).
