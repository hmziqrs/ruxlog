//! Error handling for the authentication module

use crate::db::errors::DBError;
use crate::error::{ErrorCode, ErrorResponse, response::IntoErrorResponse};
use crate::services::auth::AuthError;

/// Implementation of IntoErrorResponse for the AuthError type
impl IntoErrorResponse for AuthError {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            AuthError::InvalidCredentials => {
                ErrorResponse::new(ErrorCode::InvalidCredentials)
            },
            AuthError::PasswordVerificationError => {
                // Map to the same error code as invalid credentials for security
                // (don't reveal if the user exists or not)
                ErrorResponse::new(ErrorCode::InvalidCredentials)
            },
            AuthError::UserNotFound => {
                // In APIs, we might want to map this to InvalidCredentials as well
                // to avoid leaking information about existing users
                ErrorResponse::new(ErrorCode::UserNotFound)
            },
            AuthError::Unauthorized => {
                ErrorResponse::new(ErrorCode::Unauthorized)
            },
            AuthError::SessionExpired => {
                ErrorResponse::new(ErrorCode::SessionExpired)
            },
            AuthError::DatabaseError(db_err) => {
                // Map database errors to appropriate error codes
                match db_err {
                    DBError::ConnectionError(err) => {
                        ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                            .with_details(format!("DB connection error: {}", err))
                    },
                    DBError::QueryError(err) => {
                        if let diesel::result::Error::NotFound = err {
                            ErrorResponse::new(ErrorCode::RecordNotFound)
                        } else {
                            ErrorResponse::new(ErrorCode::QueryError)
                                .with_details(format!("DB query error: {}", err))
                        }
                    },
                    DBError::UnknownError(err) => {
                        ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                            .with_details(format!("Unknown DB error: {}", err))
                    },
                    DBError::PasswordHashError => {
                        ErrorResponse::new(ErrorCode::InternalServerError)
                            .with_details("Password hashing error")
                    },
                }
            },
            AuthError::InternalError(err) => {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_details(err)
            },
        }
    }
}

/// Implementation of From<AuthError> for ErrorResponse
impl From<AuthError> for ErrorResponse {
    fn from(err: AuthError) -> Self {
        err.into_error_response()
    }
}