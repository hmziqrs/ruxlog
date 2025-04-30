//! Error handling for database operations

use sea_orm::DbErr;
use crate::error::{ErrorCode, ErrorResponse, IntoErrorResponse};

/// Standardized handling for SeaORM database errors
impl IntoErrorResponse for DbErr {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            // Connection errors
            DbErr::Conn(err) => {
                ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                    .with_message("Database connection error")
                    .with_details(err.to_string())
            },
            
            // SQL Query errors
            DbErr::Exec(err) => {
                ErrorResponse::new(ErrorCode::QueryError)
                    .with_message("Error executing database query")
                    .with_details(err.to_string())
            },
            
            // Query builder errors
            DbErr::Query(err) => {
                ErrorResponse::new(ErrorCode::QueryError)
                    .with_message("Error building database query")
                    .with_details(err.to_string())
            },
            
            // Record not found
            DbErr::RecordNotFound(err) => {
                ErrorResponse::new(ErrorCode::RecordNotFound)
                    .with_message("Record not found")
                    .with_details(err.to_string())
            },
            
            // Custom errors
            DbErr::Custom(err) => {
                ErrorResponse::new(ErrorCode::QueryError)
                    .with_message("Database error")
                    .with_details(err.to_string())
            },
            
            // Type conversion errors
            DbErr::Type(err) => {
                ErrorResponse::new(ErrorCode::InvalidValue)
                    .with_message("Type conversion error")
                    .with_details(err.to_string())
            },
            
            // JSON errors
            DbErr::Json(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("JSON serialization error")
                    .with_details(err.to_string())
            },
            
            // Migration errors
            DbErr::Migration(err) => {
                ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                    .with_message("Database migration error")
                    .with_details(err.to_string())
            },
            
            // TxIsolationLevel errors
            // #[cfg(feature = "sea-orm-active-enums")]
            // DbErr::TxIsolationLevel(err) => {
            //     ErrorResponse::new(ErrorCode::TransactionError)
            //         .with_message("Transaction isolation level error")
            //         .with_details(err.to_string())
            // },
            
            // Pool error
            // #[cfg(feature = "sea-orm-active-enums")]
            // DbErr::PoolTimedOut(err) => {
            //     ErrorResponse::new(ErrorCode::DatabaseConnectionError)
            //         .with_message("Database connection pool timeout")
            //         .with_details(err.to_string())
            // },
            
            // Catch-all for other errors
            _ => {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message("Unknown database error")
                    .with_details(self.to_string())
            }
        }
    }
}

/// Implement From<DbErr> for ErrorResponse for convenience
impl From<DbErr> for ErrorResponse {
    fn from(err: DbErr) -> Self {
        err.into_error_response()
    }
}

/// Represents the result of a database operation
pub type DbResult<T> = Result<T, ErrorResponse>;

/// Database-specific error handling utilities
pub trait DbResultExt<T> {
    /// Convert a Result<T, DbErr> to a Result<T, ErrorResponse>
    fn map_err_to_response(self) -> DbResult<T>;
    
    /// Handle the not found case with a custom message
    fn not_found_with_message(self, message: &str) -> DbResult<T>;
}

impl<T> DbResultExt<T> for Result<T, DbErr> {
    fn map_err_to_response(self) -> DbResult<T> {
        self.map_err(Into::into)
    }
    
    fn not_found_with_message(self, message: &str) -> DbResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(DbErr::RecordNotFound(_)) => {
                Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                    .with_message(message))
            },
            Err(err) => Err(err.into()),
        }
    }
}