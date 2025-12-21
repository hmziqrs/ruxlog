use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// New ban record to be created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserBan {
    pub user_id: i32,
    pub reason: Option<String>,
    pub banned_by: Option<i32>,
    pub expires_at: Option<DateTimeWithTimeZone>,
}

impl NewUserBan {
    pub fn new(user_id: i32) -> Self {
        Self {
            user_id,
            reason: None,
            banned_by: None,
            expires_at: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn with_banned_by(mut self, admin_id: i32) -> Self {
        self.banned_by = Some(admin_id);
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTimeWithTimeZone) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn permanent(self) -> Self {
        Self {
            expires_at: None,
            ..self
        }
    }
}

/// Query params for listing bans
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserBanQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub active_only: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
