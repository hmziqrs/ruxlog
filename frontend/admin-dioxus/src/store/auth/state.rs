use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::OnceLock;

use crate::store::media::Media;
use crate::store::StateFrame;

pub struct AuthState {
    pub user: GlobalSignal<Option<AuthUser>>,

    pub login_status: GlobalSignal<StateFrame>,
    pub logout_status: GlobalSignal<StateFrame>,

    pub init_status: GlobalSignal<StateFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    // #[serde(rename = "super-admin")]
    SuperAdmin,
    // #[serde(rename = "admin")]
    Admin,
    // #[serde(rename = "moderator")]
    Moderator,
    // #[serde(rename = "author")]
    Author,
    // #[serde(rename = "user")]
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

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UserRole::from_str(s.to_lowercase().as_str())
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        role.to_i32()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: Option<Media>,
    pub is_verified: bool,
    pub role: UserRole,
}

impl AuthUser {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

static AUTH_STATE: OnceLock<AuthState> = OnceLock::new();

pub fn use_auth() -> &'static AuthState {
    AUTH_STATE.get_or_init(|| AuthState::new())
}
