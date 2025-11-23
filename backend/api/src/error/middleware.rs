use axum::response::IntoResponse;
use serde_json::{json, Value};

use crate::{
    db::sea_models::user::UserRole,
    error::{ErrorCode, ErrorResponse, IntoErrorResponse},
};

/// Errors originating from the CSRF middleware
#[derive(Debug, thiserror::Error)]
pub enum CsrfError {
    #[error("CSRF token missing")]
    MissingToken,
    #[error("CSRF token header is invalid")]
    InvalidHeader,
    #[error("CSRF token base64 decoding failed")]
    InvalidBase64,
    #[error("CSRF token UTF-8 decoding failed")]
    InvalidUtf8,
    #[error("CSRF token mismatch")]
    TokenMismatch,
}

impl IntoErrorResponse for CsrfError {
    fn into_error_response(self) -> ErrorResponse {
        let base = ErrorResponse::new(ErrorCode::InvalidToken)
            .with_message("Request is from an unverified client");

        match self {
            Self::MissingToken => base.with_context(json!({ "reason": "missing" })),
            Self::InvalidHeader => base
                .with_context(json!({ "reason": "invalid_header" }))
                .with_details("Failed to read csrf-token header as string"),
            Self::InvalidBase64 => base
                .with_context(json!({ "reason": "invalid_base64" }))
                .with_details("Failed to decode csrf-token header"),
            Self::InvalidUtf8 => base
                .with_context(json!({ "reason": "invalid_utf8" }))
                .with_details("Decoded csrf-token was not valid UTF-8"),
            Self::TokenMismatch => base.with_context(json!({ "reason": "mismatch" })),
        }
    }
}

impl From<CsrfError> for ErrorResponse {
    fn from(err: CsrfError) -> Self {
        err.into_error_response()
    }
}

impl IntoResponse for CsrfError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse::from(self).into_response()
    }
}

/// Errors originating from the CORS configuration / checks.
#[derive(Debug, thiserror::Error)]
pub enum CorsError {
    #[error("Origin not allowed: {origin}")]
    OriginNotAllowed { origin: String },
    #[error("Invalid Origin header")]
    InvalidOriginHeader,
}

impl IntoErrorResponse for CorsError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            Self::OriginNotAllowed { origin } => ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("CORS origin not allowed")
                .with_context(json!({ "origin": origin })),
            Self::InvalidOriginHeader => ErrorResponse::new(ErrorCode::InvalidFormat)
                .with_message("Invalid Origin header")
                .with_context(json!({ "header": "Origin" })),
        }
    }
}

impl From<CorsError> for ErrorResponse {
    fn from(err: CorsError) -> Self {
        err.into_error_response()
    }
}

impl IntoResponse for CorsError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse::from(self).into_response()
    }
}

/// Errors emitted by the role-based permission middleware.
#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("No authenticated user")]
    NoUser,
    #[error("Insufficient role for the requested resource")]
    InsufficientRole {
        required: UserRole,
        actual: UserRole,
    },
}

impl IntoErrorResponse for PermissionError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            Self::NoUser => ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("You must be logged in to access this resource"),
            Self::InsufficientRole { required, actual } => {
                ErrorResponse::new(ErrorCode::OperationNotAllowed)
                    .with_message("You don't have the required permission level")
                    .with_context(role_context(required, actual))
            }
        }
    }
}

impl From<PermissionError> for ErrorResponse {
    fn from(err: PermissionError) -> Self {
        err.into_error_response()
    }
}

impl IntoResponse for PermissionError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse::from(self).into_response()
    }
}

fn role_context(required: UserRole, actual: UserRole) -> Value {
    json!({
        "requiredRole": required,
        "userRole": actual,
    })
}

/// Errors emitted by the `user_status` middleware family.
#[derive(Debug, thiserror::Error)]
pub enum UserStatusError {
    #[error("User has not completed verification")]
    NotVerified,
    #[error("Resource is only available to unverified users")]
    VerifiedOnly,
    #[error("Resource is only available to unauthenticated users")]
    AuthenticatedOnly,
    #[error("Resource is only available to authenticated users")]
    UnauthenticatedOnly,
}

impl IntoErrorResponse for UserStatusError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            Self::NotVerified => ErrorResponse::new(ErrorCode::EmailVerificationRequired)
                .with_message("User not verified")
                .with_context(json!({ "gate": "only_verified" })),
            Self::VerifiedOnly => ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("Resource not available")
                .with_context(json!({ "gate": "only_unverified" })),
            Self::AuthenticatedOnly => ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("Resource not available")
                .with_context(json!({ "gate": "only_unauthenticated" })),
            Self::UnauthenticatedOnly => ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Resource not available")
                .with_context(json!({ "gate": "only_authenticated" })),
        }
    }
}

impl From<UserStatusError> for ErrorResponse {
    fn from(err: UserStatusError) -> Self {
        err.into_error_response()
    }
}

impl IntoResponse for UserStatusError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse::from(self).into_response()
    }
}

/// Errors emitted by the dynamic route blocker middleware.
#[derive(Debug, thiserror::Error)]
pub enum RouteBlockerError {
    #[error("Route blocked by admin policy: {path}")]
    Blocked { path: String },
    #[error("Failed to check route blocker status: {0}")]
    CheckFailed(String),
}

impl IntoErrorResponse for RouteBlockerError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            Self::Blocked { path } => ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("This route is currently unavailable")
                .with_context(json!({ "path": path })),
            Self::CheckFailed(error) => ErrorResponse::new(ErrorCode::ServiceUnavailable)
                .with_message("Failed to verify route availability")
                .with_details(error),
        }
    }
}

impl From<RouteBlockerError> for ErrorResponse {
    fn from(err: RouteBlockerError) -> Self {
        err.into_error_response()
    }
}

impl IntoResponse for RouteBlockerError {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse::from(self).into_response()
    }
}
