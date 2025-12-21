//! Core traits for authentication integration

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::error::AuthError;

/// Ban status for a user
#[derive(Debug, Clone)]
pub enum BanStatus {
    /// User is not banned
    NotBanned,
    /// User is banned with optional reason and expiry
    Banned {
        reason: Option<String>,
        expires_at: Option<DateTime<FixedOffset>>,
        banned_by: Option<i64>,
    },
}

impl BanStatus {
    /// Check if the user is currently banned
    pub fn is_banned(&self) -> bool {
        match self {
            Self::NotBanned => false,
            Self::Banned { expires_at, .. } => {
                if let Some(expires) = expires_at {
                    chrono::Utc::now().fixed_offset() < *expires
                } else {
                    true // Permanent ban
                }
            }
        }
    }
}

/// Trait for user models
///
/// Implement this on your user entity to integrate with rux-auth.
pub trait AuthUser: Clone + Debug + Send + Sync + 'static {
    /// The user ID type (e.g., i32, i64, Uuid)
    type Id: Clone + Debug + Send + Sync + Serialize + DeserializeOwned + PartialEq + 'static;

    /// Get the user's unique identifier
    fn id(&self) -> Self::Id;

    /// Hash used to invalidate sessions when password changes
    ///
    /// Return password hash bytes for password users, or email bytes for OAuth users.
    fn session_auth_hash(&self) -> &[u8];

    /// Whether the user's email is verified
    fn email_verified(&self) -> bool;

    /// Whether the user has TOTP enabled
    fn totp_enabled(&self) -> bool;

    /// The user's role level for hierarchical permission checks
    ///
    /// Higher numbers = more permissions (e.g., User=0, Admin=3, SuperAdmin=4)
    fn role_level(&self) -> i32;
}

/// Backend trait for fetching user data and performing auth operations
///
/// Implement this to connect rux-auth to your database/services.
#[async_trait]
pub trait AuthBackend: Clone + Send + Sync + 'static {
    /// The user type returned by this backend
    type User: AuthUser;

    /// Fetch a user by their ID
    ///
    /// Returns `None` if user doesn't exist.
    async fn get_user(
        &self,
        id: &<Self::User as AuthUser>::Id,
    ) -> Result<Option<Self::User>, AuthError>;

    /// Check if a user is banned
    ///
    /// Called when `not_banned()` requirement is set.
    async fn check_ban(
        &self,
        user_id: &<Self::User as AuthUser>::Id,
    ) -> Result<BanStatus, AuthError>;

    /// Verify a password for re-authentication
    ///
    /// Called when `reauth_within()` requirement needs password confirmation.
    async fn verify_password(
        &self,
        user_id: &<Self::User as AuthUser>::Id,
        password: &str,
    ) -> Result<bool, AuthError>;

    /// Called after successful login (optional hook)
    async fn on_login(&self, _user: &Self::User) -> Result<(), AuthError> {
        Ok(())
    }

    /// Called after logout (optional hook)
    async fn on_logout(&self, _user_id: &<Self::User as AuthUser>::Id) -> Result<(), AuthError> {
        Ok(())
    }
}
