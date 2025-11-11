use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// New session record to be created when a user logs in (or a device is registered).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserSession {
    pub user_id: i32,
    pub device: Option<String>,
    pub ip_address: Option<String>,
}

impl NewUserSession {
    pub fn new(user_id: i32, device: Option<String>, ip_address: Option<String>) -> Self {
        Self {
            user_id,
            device,
            ip_address,
        }
    }
}

/// Update payload for a session. Used to touch last_seen or revoke a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserSession {
    pub last_seen: Option<DateTimeWithTimeZone>,
    pub revoked_at: Option<DateTimeWithTimeZone>,
}

impl UpdateUserSession {
    /// Touch the session activity. Sets last_seen to now (UTC fixed offset).
    pub fn touch() -> Self {
        let now = chrono::Utc::now().fixed_offset();
        Self {
            last_seen: Some(now),
            revoked_at: None,
        }
    }

    /// Revoke the session immediately. Sets revoked_at and last_seen to now.
    pub fn revoke() -> Self {
        let now = chrono::Utc::now().fixed_offset();
        Self {
            last_seen: Some(now),
            revoked_at: Some(now),
        }
    }
}

/// Admin query for listing/filtering user sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserSessionQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    /// Exact match filter (use contains/like behavior in actions if needed)
    pub device: Option<String>,
    /// Exact match filter (use contains/like behavior in actions if needed)
    pub ip_address: Option<String>,
    /// When true, only sessions where revoked_at IS NULL
    pub active_only: Option<bool>,
    /// Filter sessions last seen since this time
    pub seen_since: Option<DateTimeWithTimeZone>,
    /// Filter sessions revoked since this time
    pub revoked_since: Option<DateTimeWithTimeZone>,
    /// Fields: ["last_seen", "revoked_at", "user_id", "id"]
    pub sort_by: Option<Vec<String>>,
    /// "asc" | "desc" (default desc in actions)
    pub sort_order: Option<String>,
}
