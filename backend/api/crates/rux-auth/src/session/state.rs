//! Rich session state stored alongside user authentication

use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

use crate::traits::BanStatus;

/// Rich session state stored in the session store
///
/// Contains authentication metadata beyond just the user ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionState<UserId> {
    /// The authenticated user's ID
    pub user_id: UserId,

    /// When the user originally authenticated
    pub authenticated_at: DateTime<FixedOffset>,

    /// Whether email was verified at login time
    pub email_verified: bool,

    /// When TOTP was verified this session (None if not yet verified)
    pub totp_verified_at: Option<DateTime<FixedOffset>>,

    /// When password was last re-entered for sensitive operations
    pub reauthenticated_at: Option<DateTime<FixedOffset>>,

    /// Last time ban status was checked
    pub ban_checked_at: Option<DateTime<FixedOffset>>,

    /// Cached ban status
    pub is_banned: bool,

    /// Optional device identifier
    pub device: Option<String>,

    /// Optional IP address
    pub ip_address: Option<String>,

    /// Last activity timestamp
    pub last_seen: DateTime<FixedOffset>,
}

impl<UserId: Clone> AuthSessionState<UserId> {
    /// Create new session state for a user
    pub fn new(user_id: UserId, email_verified: bool) -> Self {
        let now = Utc::now().fixed_offset();
        Self {
            user_id,
            authenticated_at: now,
            email_verified,
            totp_verified_at: None,
            reauthenticated_at: None,
            ban_checked_at: None,
            is_banned: false,
            device: None,
            ip_address: None,
            last_seen: now,
        }
    }

    /// Set device and IP metadata
    pub fn with_metadata(mut self, device: Option<String>, ip_address: Option<String>) -> Self {
        self.device = device;
        self.ip_address = ip_address;
        self
    }

    /// Mark TOTP as verified for this session
    pub fn mark_totp_verified(&mut self) {
        self.totp_verified_at = Some(Utc::now().fixed_offset());
    }

    /// Mark as recently re-authenticated
    pub fn mark_reauthenticated(&mut self) {
        self.reauthenticated_at = Some(Utc::now().fixed_offset());
    }

    /// Update ban status cache
    pub fn update_ban_status(&mut self, status: &BanStatus) {
        self.ban_checked_at = Some(Utc::now().fixed_offset());
        self.is_banned = status.is_banned();
    }

    /// Update last seen timestamp
    pub fn touch(&mut self) {
        self.last_seen = Utc::now().fixed_offset();
    }

    /// Check if TOTP was verified this session
    pub fn is_totp_verified(&self) -> bool {
        self.totp_verified_at.is_some()
    }

    /// Check if reauth was within the given duration
    pub fn reauth_within(&self, duration: Duration) -> bool {
        self.reauthenticated_at
            .map(|t| Utc::now().fixed_offset() - t < duration)
            .unwrap_or(false)
    }

    /// Check if ban cache is stale (older than max_age)
    pub fn ban_cache_stale(&self, max_age: Duration) -> bool {
        self.ban_checked_at
            .map(|t| Utc::now().fixed_offset() - t > max_age)
            .unwrap_or(true)
    }

    /// Refresh email verification status from user
    pub fn refresh_verification(&mut self, email_verified: bool) {
        self.email_verified = email_verified;
    }
}
