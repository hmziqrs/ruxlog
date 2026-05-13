//! Error handling for database operations

use crate::error::{ErrorCode, ErrorResponse, IntoErrorResponse};
use sea_orm::DbErr;

/// Map SQLSTATE codes and common database error messages to ErrorCode
fn classify_db_error(msg: &str) -> ErrorCode {
    let lower = msg.to_lowercase();
    // Duplicate / unique constraint violations (Postgres 23505)
    if msg.contains("23505")
        || lower.contains("duplicate key value")
        || lower.contains("unique constraint")
    {
        return ErrorCode::DuplicateEntry;
    }
    // Foreign key violations (Postgres 23503)
    if msg.contains("23503") || lower.contains("violates foreign key constraint") {
        return ErrorCode::IntegrityError;
    }
    // Not-null violations (Postgres 23502)
    if msg.contains("23502")
        || lower.contains("not-null constraint")
        || lower.contains("null value in column")
    {
        return ErrorCode::IntegrityError;
    }
    // Check constraint (Postgres 23514) and other integrity issues (class 23*)
    if msg.contains("23514")
        || lower.contains("check constraint")
        || msg.contains("23P01")
        || lower.contains("exclusion constraint")
    {
        return ErrorCode::IntegrityError;
    }
    // Deadlock detected (Postgres 40P01)
    if msg.contains("40P01") || lower.contains("deadlock detected") {
        return ErrorCode::TransactionError;
    }
    // Serialization failure (Postgres 40001)
    if msg.contains("40001")
        || lower.contains("could not serialize access due to")
        || lower.contains("serialization failure")
    {
        return ErrorCode::TransactionError;
    }
    // Default
    ErrorCode::QueryError
}

/// Standardized handling for SeaORM database errors
impl IntoErrorResponse for DbErr {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            DbErr::Conn(err) => ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                .with_message("Database connection error")
                .with_details(err.to_string()),

            DbErr::Exec(err) => {
                let msg = err.to_string();
                let code = classify_db_error(&msg);
                if code == ErrorCode::QueryError {
                    ErrorResponse::new(ErrorCode::QueryError)
                        .with_message("Error executing database query")
                        .with_details(msg)
                } else {
                    let friendly = match code {
                        ErrorCode::DuplicateEntry => "Duplicate entry",
                        ErrorCode::IntegrityError => "Integrity constraint violation",
                        ErrorCode::TransactionError => "Transaction error",
                        _ => "Database error",
                    };
                    ErrorResponse::new(code)
                        .with_message(friendly)
                        .with_details(msg)
                }
            }

            DbErr::Query(err) => {
                let msg = err.to_string();
                let code = classify_db_error(&msg);
                if code == ErrorCode::QueryError {
                    ErrorResponse::new(ErrorCode::QueryError)
                        .with_message("Error building database query")
                        .with_details(msg)
                } else {
                    let friendly = match code {
                        ErrorCode::DuplicateEntry => "Duplicate entry",
                        ErrorCode::IntegrityError => "Integrity constraint violation",
                        ErrorCode::TransactionError => "Transaction error",
                        _ => "Database error",
                    };
                    ErrorResponse::new(code)
                        .with_message(friendly)
                        .with_details(msg)
                }
            }

            DbErr::RecordNotFound(err) => ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Record not found")
                .with_details(err.to_string()),

            DbErr::Custom(err) => ErrorResponse::new(ErrorCode::QueryError)
                .with_message("Database error")
                .with_details(err.to_string()),

            DbErr::Type(err) => ErrorResponse::new(ErrorCode::InvalidValue)
                .with_message("Type conversion error")
                .with_details(err.to_string()),

            DbErr::Json(err) => ErrorResponse::new(ErrorCode::InvalidFormat)
                .with_message("JSON serialization error")
                .with_details(err.to_string()),

            DbErr::Migration(err) => ErrorResponse::new(ErrorCode::DatabaseConnectionError)
                .with_message("Database migration error")
                .with_details(err.to_string()),

            // TxIsolationLevel errors
            // #[cfg(feature = "sea-orm-active-enums")]
            // },

            // Pool error
            // #[cfg(feature = "sea-orm-active-enums")]
            // },
            _ => ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Unknown database error")
                .with_details(self.to_string()),
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
#[allow(clippy::result_large_err)]
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
                Err(ErrorResponse::new(ErrorCode::RecordNotFound).with_message(message))
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_duplicate_key_23505() {
        assert_eq!(
            classify_db_error("ERROR: duplicate key value violates unique constraint \"idx_email\" (SQLSTATE 23505)"),
            ErrorCode::DuplicateEntry
        );
    }

    #[test]
    fn test_classify_duplicate_key_lowercase_message() {
        assert_eq!(
            classify_db_error("duplicate key value violates unique constraint"),
            ErrorCode::DuplicateEntry
        );
    }

    #[test]
    fn test_classify_unique_constraint() {
        assert_eq!(
            classify_db_error("unique constraint violation for column slug"),
            ErrorCode::DuplicateEntry
        );
    }

    #[test]
    fn test_classify_foreign_key_23503() {
        assert_eq!(
            classify_db_error("ERROR: insert or update on table \"posts\" violates foreign key constraint \"fk_author\" (SQLSTATE 23503)"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_foreign_key_lowercase() {
        assert_eq!(
            classify_db_error("violates foreign key constraint"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_not_null_23502() {
        assert_eq!(
            classify_db_error("ERROR: null value in column \"title\" violates not-null constraint (SQLSTATE 23502)"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_not_null_lowercase() {
        assert_eq!(
            classify_db_error("not-null constraint violation"),
            ErrorCode::IntegrityError
        );
        assert_eq!(
            classify_db_error("null value in column \"email\""),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_check_constraint_23514() {
        assert_eq!(
            classify_db_error("ERROR: new row for relation \"users\" violates check constraint \"ck_email_format\" (SQLSTATE 23514)"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_check_constraint_lowercase() {
        assert_eq!(
            classify_db_error("check constraint \"ck_positive\" violated"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_exclusion_constraint_23p01() {
        assert_eq!(
            classify_db_error("ERROR: conflicting key value violates exclusion constraint (23P01)"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_exclusion_constraint_lowercase() {
        assert_eq!(
            classify_db_error("exclusion constraint violation detected"),
            ErrorCode::IntegrityError
        );
    }

    #[test]
    fn test_classify_deadlock_40p01() {
        assert_eq!(
            classify_db_error("ERROR: deadlock detected (SQLSTATE 40P01)"),
            ErrorCode::TransactionError
        );
    }

    #[test]
    fn test_classify_deadlock_lowercase() {
        assert_eq!(
            classify_db_error("deadlock detected while waiting for lock"),
            ErrorCode::TransactionError
        );
    }

    #[test]
    fn test_classify_serialization_failure_40001() {
        assert_eq!(
            classify_db_error("ERROR: could not serialize access due to concurrent update (SQLSTATE 40001)"),
            ErrorCode::TransactionError
        );
    }

    #[test]
    fn test_classify_serialization_failure_lowercase() {
        assert_eq!(
            classify_db_error("serialization failure"),
            ErrorCode::TransactionError
        );
    }

    #[test]
    fn test_classify_unknown_error() {
        assert_eq!(
            classify_db_error("some random database error"),
            ErrorCode::QueryError
        );
    }

    #[test]
    fn test_classify_empty_message() {
        assert_eq!(classify_db_error(""), ErrorCode::QueryError);
    }
}
