//! Error handling for the authentication module

use crate::error::{response::IntoErrorResponse, ErrorCode, ErrorResponse};
use crate::services::auth::AuthError;

/// Implementation of IntoErrorResponse for the AuthError type
impl IntoErrorResponse for AuthError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            AuthError::InvalidCredentials => ErrorResponse::new(ErrorCode::InvalidCredentials),
            AuthError::PasswordVerificationError => {
                // Map to the same error code as invalid credentials for security
                ErrorResponse::new(ErrorCode::InvalidCredentials)
            }
            AuthError::UserNotFound => {
                // In APIs, we might want to map this to InvalidCredentials as well
                ErrorResponse::new(ErrorCode::InvalidCredentials)
            }
            AuthError::Unauthorized => ErrorResponse::new(ErrorCode::Unauthorized),
            AuthError::SessionExpired => ErrorResponse::new(ErrorCode::SessionExpired),
            AuthError::DatabaseError(err_resp) => err_resp,
            AuthError::InternalError(err) => {
                ErrorResponse::new(ErrorCode::InternalServerError).with_details(err)
            }
        }
    }
}

/// Implementation of From<AuthError> for ErrorResponse
impl From<AuthError> for ErrorResponse {
    fn from(err: AuthError) -> Self {
        err.into_error_response()
    }
}
