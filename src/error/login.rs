//! Error handling for axum_login errors

use crate::error::{ErrorCode, ErrorResponse};
use crate::services::auth::AuthBackend;

/// Implementation of From<axum_login::Error<AuthBackend>> for ErrorResponse
impl From<axum_login::Error<AuthBackend>> for ErrorResponse {
    fn from(err: axum_login::Error<AuthBackend>) -> Self {
        eprintln!("axum_login error: {:?}", err);

        ErrorResponse::new(ErrorCode::SessionExpired)
            .with_details(format!("axum_login error: {:?}", err))
    }
}
