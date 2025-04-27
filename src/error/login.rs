//! Error handling for axum_login errors

use crate::error::{ErrorCode, ErrorResponse};
use crate::services::auth::AuthBackend;

/// Implementation of From<axum_login::Error<AuthBackend>> for ErrorResponse
impl From<axum_login::Error<AuthBackend>> for ErrorResponse {
    fn from(err: axum_login::Error<AuthBackend>) -> Self {
        // Log the error
        eprintln!("axum_login error: {:?}", err);
        
        // Return a generic session error
        ErrorResponse::new(ErrorCode::SessionExpired)
            .with_details(format!("axum_login error: {:?}", err))
    }
}