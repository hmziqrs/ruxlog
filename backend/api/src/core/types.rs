use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub usage_count: Option<i64>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Password verification failed")]
    PasswordVerificationError,

    #[error("Internal authentication error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum TagError {
    #[error("Failed to load tags: {0}")]
    LoadFailed(String),
}

