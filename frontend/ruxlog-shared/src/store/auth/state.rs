use chrono::{DateTime, Utc};
use dioxus::prelude::*;
pub use ruxlog_types::enums::UserRole;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::store::Media;
use oxstore::StateFrame;

pub struct AuthState {
    pub user: GlobalSignal<Option<AuthUser>>,

    pub login_status: GlobalSignal<StateFrame>,
    pub logout_status: GlobalSignal<StateFrame>,
    pub register_status: GlobalSignal<StateFrame>,

    pub init_status: GlobalSignal<StateFrame>,
    pub two_factor: GlobalSignal<StateFrame<Option<TwoFactorSetup>>>,
    pub sessions: GlobalSignal<StateFrame<Vec<UserSession>>>,

    /// Two-step 2FA-at-login (F#4/F#7/F#16). When `login()` receives a
    /// `{ status: "totp_required", totp_token }` response (the user has 2FA
    /// enrolled, so a correct password is NOT enough for a full session), the
    /// opaque pending token is stored here as `data`. The login screen then
    /// shows a TOTP code input; `verify_login_totp(code)` exchanges the token
    /// + code at `/auth/v1/login/totp` for the full session and clears this.
    /// `data == None` (or a non-success frame) means no TOTP step is pending.
    pub login_totp: GlobalSignal<StateFrame<String>>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwoFactorVerifyPayload {
    pub code: String,
}

/// Second step of the two-step 2FA-at-login flow. `totp_token` is the opaque
/// pending credential from `login()`'s `totp_required` response; `code` is the
/// 6-digit TOTP code from the user's authenticator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginTotpPayload {
    pub totp_token: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSession {
    pub id: String,
    pub user_id: i32,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

static AUTH_STATE: OnceLock<AuthState> = OnceLock::new();

pub fn use_auth() -> &'static AuthState {
    AUTH_STATE.get_or_init(|| AuthState::new())
}
