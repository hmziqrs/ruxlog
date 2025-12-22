//! Error handling for the authentication module

use crate::error::{response::IntoErrorResponse, ErrorCode, ErrorResponse};
use rux_auth::{AuthError, AuthErrorCode};

/// Implementation of IntoErrorResponse for the rux_auth::AuthError type
impl IntoErrorResponse for AuthError {
    fn into_error_response(self) -> ErrorResponse {
        match self.code() {
            AuthErrorCode::Unauthenticated => ErrorResponse::new(ErrorCode::Unauthorized),
            AuthErrorCode::AlreadyAuthenticated => {
                ErrorResponse::new(ErrorCode::OperationNotAllowed)
                    .with_message("Already authenticated")
            }
            AuthErrorCode::InvalidCredentials => ErrorResponse::new(ErrorCode::InvalidCredentials),
            AuthErrorCode::SessionExpired => ErrorResponse::new(ErrorCode::SessionExpired),
            AuthErrorCode::SessionError => ErrorResponse::new(ErrorCode::InternalServerError),
            AuthErrorCode::VerificationRequired => {
                ErrorResponse::new(ErrorCode::EmailVerificationRequired)
            }
            AuthErrorCode::AlreadyVerified => ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("Already verified"),
            AuthErrorCode::TotpRequired => ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Two-factor authentication required"),
            AuthErrorCode::TotpInvalid => {
                ErrorResponse::new(ErrorCode::InvalidToken).with_message("Invalid 2FA code")
            }
            AuthErrorCode::ReauthRequired => ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Password confirmation required"),
            AuthErrorCode::Banned => {
                ErrorResponse::new(ErrorCode::AccountLocked).with_message("Account banned")
            }
            AuthErrorCode::InsufficientRole => ErrorResponse::new(ErrorCode::OperationNotAllowed),
            AuthErrorCode::PermissionDenied => ErrorResponse::new(ErrorCode::OperationNotAllowed),
            AuthErrorCode::OAuthError => ErrorResponse::new(ErrorCode::ExternalServiceError),
            AuthErrorCode::CsrfInvalid => ErrorResponse::new(ErrorCode::InvalidToken),
            AuthErrorCode::BackendError => ErrorResponse::new(ErrorCode::InternalServerError),
            AuthErrorCode::InternalError => ErrorResponse::new(ErrorCode::InternalServerError),
        }
    }
}

/// Implementation of From<AuthError> for ErrorResponse
impl From<AuthError> for ErrorResponse {
    fn from(err: AuthError) -> Self {
        err.into_error_response()
    }
}
