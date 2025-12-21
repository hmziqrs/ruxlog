//! Authentication error types

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error codes for authentication/authorization failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthErrorCode {
    /// User must be logged in
    #[serde(rename = "AUTH_UNAUTHENTICATED")]
    Unauthenticated,

    /// User is already logged in (for login/register routes)
    #[serde(rename = "AUTH_ALREADY_AUTHENTICATED")]
    AlreadyAuthenticated,

    /// Invalid credentials provided
    #[serde(rename = "AUTH_INVALID_CREDENTIALS")]
    InvalidCredentials,

    /// Session has expired
    #[serde(rename = "AUTH_SESSION_EXPIRED")]
    SessionExpired,

    /// Session error (storage/retrieval)
    #[serde(rename = "AUTH_SESSION_ERROR")]
    SessionError,

    /// Email verification required
    #[serde(rename = "AUTH_VERIFICATION_REQUIRED")]
    VerificationRequired,

    /// User already verified (for verification routes)
    #[serde(rename = "AUTH_ALREADY_VERIFIED")]
    AlreadyVerified,

    /// Two-factor authentication required
    #[serde(rename = "AUTH_TOTP_REQUIRED")]
    TotpRequired,

    /// Invalid TOTP code
    #[serde(rename = "AUTH_TOTP_INVALID")]
    TotpInvalid,

    /// Password re-entry required for sensitive action
    #[serde(rename = "AUTH_REAUTH_REQUIRED")]
    ReauthRequired,

    /// Account is banned
    #[serde(rename = "AUTH_BANNED")]
    Banned,

    /// Insufficient role level
    #[serde(rename = "AUTH_INSUFFICIENT_ROLE")]
    InsufficientRole,

    /// Permission denied
    #[serde(rename = "AUTH_PERMISSION_DENIED")]
    PermissionDenied,

    /// OAuth provider error
    #[serde(rename = "AUTH_OAUTH_ERROR")]
    OAuthError,

    /// Invalid CSRF token
    #[serde(rename = "AUTH_CSRF_INVALID")]
    CsrfInvalid,

    /// Backend error
    #[serde(rename = "AUTH_BACKEND_ERROR")]
    BackendError,

    /// Internal error
    #[serde(rename = "AUTH_INTERNAL_ERROR")]
    InternalError,
}

impl AuthErrorCode {
    /// Returns the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::AlreadyAuthenticated => StatusCode::CONFLICT,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::SessionError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::VerificationRequired => StatusCode::FORBIDDEN,
            Self::AlreadyVerified => StatusCode::CONFLICT,
            Self::TotpRequired => StatusCode::FORBIDDEN,
            Self::TotpInvalid => StatusCode::UNAUTHORIZED,
            Self::ReauthRequired => StatusCode::FORBIDDEN,
            Self::Banned => StatusCode::FORBIDDEN,
            Self::InsufficientRole => StatusCode::FORBIDDEN,
            Self::PermissionDenied => StatusCode::FORBIDDEN,
            Self::OAuthError => StatusCode::BAD_GATEWAY,
            Self::CsrfInvalid => StatusCode::UNAUTHORIZED,
            Self::BackendError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the default message for this error
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "Authentication required",
            Self::AlreadyAuthenticated => "Already authenticated",
            Self::InvalidCredentials => "Invalid credentials",
            Self::SessionExpired => "Session expired",
            Self::SessionError => "Session error",
            Self::VerificationRequired => "Email verification required",
            Self::AlreadyVerified => "Already verified",
            Self::TotpRequired => "Two-factor authentication required",
            Self::TotpInvalid => "Invalid two-factor code",
            Self::ReauthRequired => "Password confirmation required",
            Self::Banned => "Account banned",
            Self::InsufficientRole => "Insufficient permissions",
            Self::PermissionDenied => "Permission denied",
            Self::OAuthError => "OAuth provider error",
            Self::CsrfInvalid => "Invalid CSRF token",
            Self::BackendError => "Backend error",
            Self::InternalError => "Internal error",
        }
    }
}

/// Authentication error with code, message, and optional context
#[derive(Debug, Clone, Serialize)]
pub struct AuthError {
    #[serde(rename = "code")]
    pub error_code: AuthErrorCode,
    pub message: String,
    #[serde(skip)]
    pub status: StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl AuthError {
    /// Create a new error with default message
    pub fn new(code: AuthErrorCode) -> Self {
        Self {
            message: code.default_message().to_string(),
            status: code.status_code(),
            error_code: code,
            context: None,
        }
    }

    /// Set a custom message
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    /// Add context data
    pub fn with_context<V: Serialize>(mut self, key: &str, value: V) -> Self {
        let ctx = self
            .context
            .get_or_insert_with(|| serde_json::json!({}));
        if let Some(obj) = ctx.as_object_mut() {
            if let Ok(v) = serde_json::to_value(value) {
                obj.insert(key.to_string(), v);
            }
        }
        self
    }

    /// Get the error code
    pub fn code(&self) -> AuthErrorCode {
        self.error_code
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_code, self.message)
    }
}

impl std::error::Error for AuthError {}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = serde_json::json!({
            "error": {
                "code": self.error_code,
                "message": self.message,
                "context": self.context,
            }
        });
        (status, axum::Json(body)).into_response()
    }
}

impl From<tower_sessions::session::Error> for AuthError {
    fn from(err: tower_sessions::session::Error) -> Self {
        tracing::error!(error = ?err, "Session error");
        Self::new(AuthErrorCode::SessionError)
    }
}
