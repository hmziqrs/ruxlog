//! Composable authentication requirements builder

use chrono::Duration;

/// Authentication requirements for route protection
///
/// Use [`auth_requirements()`] to create a new builder, then chain methods
/// to add requirements.
///
/// # Examples
///
/// ```ignore
/// // Require authenticated user
/// auth_requirements().authenticated()
///
/// // Require unauthenticated (for login page)
/// auth_requirements().unauthenticated()
///
/// // Require authenticated + verified + not banned
/// auth_requirements()
///     .authenticated()
///     .verified()
///     .not_banned()
///
/// // Require 2FA if enabled + recent password confirmation
/// auth_requirements()
///     .authenticated()
///     .totp_if_enabled()
///     .reauth_within(Duration::minutes(5))
/// ```
#[derive(Debug, Clone, Default)]
pub struct AuthRequirements {
    /// Authentication requirement: Some(true) = must be auth, Some(false) = must NOT be auth
    pub(crate) authenticated: Option<bool>,

    /// Verification requirement: must be verified
    pub(crate) verified: bool,

    /// Inverse verification: must NOT be verified
    pub(crate) unverified: bool,

    /// TOTP requirement: Some(true) = must have TOTP verified, Some(false) = TOTP if enabled
    pub(crate) totp_verified: Option<bool>,

    /// Reauth requirement: password must be confirmed within this duration
    pub(crate) reauth_within: Option<Duration>,

    /// Ban check requirement
    pub(crate) not_banned: bool,

    /// Minimum role level required
    pub(crate) min_role: Option<i32>,

    /// Ban cache duration (how long to trust cached ban status)
    pub(crate) ban_cache_duration: Duration,
}

impl AuthRequirements {
    /// Create new empty requirements
    pub fn new() -> Self {
        Self {
            ban_cache_duration: Duration::minutes(5),
            ..Default::default()
        }
    }

    /// Require the user to be authenticated
    ///
    /// Returns `Unauthenticated` error if no user is logged in.
    pub fn authenticated(mut self) -> Self {
        self.authenticated = Some(true);
        self
    }

    /// Require the user to NOT be authenticated
    ///
    /// Use this for login/register/forgot-password routes.
    /// Returns `AlreadyAuthenticated` error if a user is logged in.
    pub fn unauthenticated(mut self) -> Self {
        self.authenticated = Some(false);
        self
    }

    /// Require the user's email to be verified
    ///
    /// Returns `VerificationRequired` error if email is not verified.
    pub fn verified(mut self) -> Self {
        self.verified = true;
        self
    }

    /// Require the user's email to NOT be verified
    ///
    /// Use this for email verification routes where verified users shouldn't access.
    /// Returns `AlreadyVerified` error if email is already verified.
    pub fn unverified(mut self) -> Self {
        self.unverified = true;
        self
    }

    /// Require TOTP to be verified this session
    ///
    /// Returns `TotpRequired` error if TOTP hasn't been verified.
    /// This is strict - fails even if user doesn't have TOTP enabled.
    pub fn totp_verified(mut self) -> Self {
        self.totp_verified = Some(true);
        self
    }

    /// Require TOTP verification only if the user has TOTP enabled
    ///
    /// Returns `TotpRequired` error if user has TOTP enabled but hasn't verified this session.
    /// Passes if user doesn't have TOTP enabled.
    pub fn totp_if_enabled(mut self) -> Self {
        self.totp_verified = Some(false);
        self
    }

    /// Require recent password confirmation
    ///
    /// Returns `ReauthRequired` error if password wasn't confirmed within the duration.
    /// Use this for sensitive operations like password change or account deletion.
    pub fn reauth_within(mut self, duration: Duration) -> Self {
        self.reauth_within = Some(duration);
        self
    }

    /// Require the user to not be banned
    ///
    /// Returns `Banned` error if user has an active ban.
    /// Ban status is cached for efficiency.
    pub fn not_banned(mut self) -> Self {
        self.not_banned = true;
        self
    }

    /// Require a minimum role level
    ///
    /// Returns `InsufficientRole` error if user's role level is below the minimum.
    /// Role levels are defined by your `AuthUser::role_level()` implementation.
    pub fn role_min(mut self, level: i32) -> Self {
        self.min_role = Some(level);
        self
    }

    /// Set how long to cache ban status checks
    ///
    /// Default is 5 minutes. Set lower for stricter checking.
    pub fn ban_cache_duration(mut self, duration: Duration) -> Self {
        self.ban_cache_duration = duration;
        self
    }

    /// Check if this requires authentication
    pub fn requires_auth(&self) -> bool {
        self.authenticated == Some(true)
    }

    /// Check if this requires unauthenticated
    pub fn requires_unauth(&self) -> bool {
        self.authenticated == Some(false)
    }
}

/// Create a new authentication requirements builder
///
/// # Examples
///
/// ```ignore
/// use rux_auth::auth_requirements;
/// use chrono::Duration;
///
/// // For login route - must NOT be logged in
/// let login_requirements = auth_requirements().unauthenticated();
///
/// // For protected route - must be logged in and verified
/// let protected = auth_requirements()
///     .authenticated()
///     .verified()
///     .not_banned();
///
/// // For sensitive route - 2FA + recent password
/// let sensitive = auth_requirements()
///     .authenticated()
///     .verified()
///     .totp_if_enabled()
///     .reauth_within(Duration::minutes(5));
///
/// // For admin route - minimum role level
/// let admin = auth_requirements()
///     .authenticated()
///     .role_min(3); // Admin = 3
/// ```
pub fn auth_requirements() -> AuthRequirements {
    AuthRequirements::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_requirement() {
        let req = auth_requirements().authenticated();
        assert_eq!(req.authenticated, Some(true));
        assert!(req.requires_auth());
        assert!(!req.requires_unauth());
    }

    #[test]
    fn test_unauthenticated_requirement() {
        let req = auth_requirements().unauthenticated();
        assert_eq!(req.authenticated, Some(false));
        assert!(!req.requires_auth());
        assert!(req.requires_unauth());
    }

    #[test]
    fn test_chained_requirements() {
        let req = auth_requirements()
            .authenticated()
            .verified()
            .not_banned()
            .role_min(3);

        assert_eq!(req.authenticated, Some(true));
        assert!(req.verified);
        assert!(req.not_banned);
        assert_eq!(req.min_role, Some(3));
    }

    #[test]
    fn test_inverse_requirements() {
        let req = auth_requirements().authenticated().unverified();

        assert_eq!(req.authenticated, Some(true));
        assert!(req.unverified);
        assert!(!req.verified);
    }

    #[test]
    fn test_totp_requirements() {
        let strict = auth_requirements().totp_verified();
        assert_eq!(strict.totp_verified, Some(true));

        let conditional = auth_requirements().totp_if_enabled();
        assert_eq!(conditional.totp_verified, Some(false));
    }

    #[test]
    fn test_reauth_requirement() {
        let req = auth_requirements().reauth_within(Duration::minutes(5));
        assert!(req.reauth_within.is_some());
        assert_eq!(req.reauth_within.unwrap().num_minutes(), 5);
    }
}
