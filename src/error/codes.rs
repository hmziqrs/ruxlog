//! Error code definitions for the application
//!
//! This module defines standard error codes that can be used throughout the application.
//! Each error code has a unique string identifier that can be used for translation on the client.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard error codes for the application
///
/// Each error code has:
/// - A category (prefix)
/// - A unique identifier within that category
/// - A default message (for logging/debugging)
///
/// When sent to clients, these are serialized to strings like "AUTH_001" which can be
/// used for translation lookup on the client side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    #[serde(rename = "AUTH_001")]
    InvalidCredentials,
    #[serde(rename = "AUTH_002")]
    UserNotFound,
    #[serde(rename = "AUTH_003")]
    SessionExpired,
    #[serde(rename = "AUTH_004")]
    Unauthorized,
    #[serde(rename = "AUTH_005")]
    PasswordResetRequired,
    #[serde(rename = "AUTH_006")]
    AccountLocked,
    #[serde(rename = "AUTH_007")]
    TooManyAttempts,
    #[serde(rename = "AUTH_008")]
    EmailVerificationRequired,
    #[serde(rename = "AUTH_009")]
    InvalidToken,

    #[serde(rename = "VAL_001")]
    InvalidInput,
    #[serde(rename = "VAL_002")]
    MissingRequiredField,
    #[serde(rename = "VAL_003")]
    InvalidFormat,
    #[serde(rename = "VAL_004")]
    InvalidLength,
    #[serde(rename = "VAL_005")]
    InvalidValue,
    #[serde(rename = "VAL_006")]
    ValidationError,

    #[serde(rename = "DB_001")]
    DatabaseConnectionError,
    #[serde(rename = "DB_002")]
    RecordNotFound,
    #[serde(rename = "DB_003")]
    DuplicateEntry,
    #[serde(rename = "DB_004")]
    QueryError,
    #[serde(rename = "DB_005")]
    TransactionError,
    #[serde(rename = "DB_006")]
    RelationshipError,
    #[serde(rename = "DB_007")]
    IntegrityError,

    #[serde(rename = "SRV_001")]
    InternalServerError,
    #[serde(rename = "SRV_002")]
    ServiceUnavailable,
    #[serde(rename = "SRV_003")]
    Timeout,
    #[serde(rename = "SRV_004")]
    RateLimited,
    #[serde(rename = "SRV_005")]
    ConfigurationError,

    #[serde(rename = "BIZ_001")]
    OperationNotAllowed,
    #[serde(rename = "BIZ_002")]
    ResourceConflict,
    #[serde(rename = "BIZ_003")]
    BusinessRuleViolation,
    #[serde(rename = "BIZ_004")]
    DependencyExists,

    #[serde(rename = "EXT_001")]
    ExternalServiceError,
    #[serde(rename = "EXT_002")]
    ExternalServiceTimeout,
    #[serde(rename = "EXT_003")]
    ExternalServiceUnavailable,

    #[serde(rename = "AST_001")]
    FileUploadError,
    #[serde(rename = "AST_002")]
    FileNotFound,
    #[serde(rename = "AST_003")]
    FileTooLarge,
    #[serde(rename = "AST_004")]
    InvalidFileType,
    #[serde(rename = "AST_005")]
    StorageError,
    #[serde(rename = "AST_006")]
    FileDeletionError,
    #[serde(rename = "AST_007")]
    AssetMetadataError,

    #[serde(rename = "EML_001")]
    EmailSendingError,
    #[serde(rename = "EML_002")]
    InvalidEmailFormat,
    #[serde(rename = "EML_003")]
    EmailDeliveryError,

    #[serde(rename = "PST_001")]
    PostNotFound,
    #[serde(rename = "PST_002")]
    InvalidPostStatus,
    #[serde(rename = "PST_003")]
    PostAlreadyPublished,
    #[serde(rename = "PST_004")]
    SlugAlreadyExists,

    #[serde(rename = "CAT_001")]
    CategoryNotFound,
    #[serde(rename = "CAT_002")]
    CategoryInUse,
    #[serde(rename = "CAT_003")]
    InvalidCategoryParent,

    #[serde(rename = "TAG_001")]
    TagNotFound,
    #[serde(rename = "TAG_002")]
    TagAlreadyExists,
}

impl ErrorCode {
    /// Returns the default error message for this error code
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "Invalid username or password",
            Self::UserNotFound => "User not found",
            Self::SessionExpired => "Your session has expired, please login again",
            Self::Unauthorized => "You are not authorized to perform this action",
            Self::PasswordResetRequired => "Password reset is required",
            Self::AccountLocked => "Your account has been locked",
            Self::TooManyAttempts => "Too many attempts, please try again later",
            Self::EmailVerificationRequired => "Email verification is required",
            Self::InvalidToken => "The provided token is invalid or expired",

            Self::InvalidInput => "The provided input is invalid",
            Self::MissingRequiredField => "A required field is missing",
            Self::InvalidFormat => "The provided value has an invalid format",
            Self::InvalidLength => "The provided value has an invalid length",
            Self::InvalidValue => "The provided value is invalid",
            Self::ValidationError => "Validation error occurred",

            Self::DatabaseConnectionError => "Could not connect to the database",
            Self::RecordNotFound => "The requested record was not found",
            Self::DuplicateEntry => "A record with this value already exists",
            Self::QueryError => "There was an error executing your request",
            Self::TransactionError => "Transaction failed",
            Self::RelationshipError => "Error with relationship between records",
            Self::IntegrityError => "Database integrity constraint violation",

            Self::InternalServerError => "An internal server error occurred",
            Self::ServiceUnavailable => "The service is currently unavailable",
            Self::Timeout => "The request timed out",
            Self::RateLimited => "Too many requests, please try again later",
            Self::ConfigurationError => "Server configuration error",

            Self::OperationNotAllowed => "This operation is not allowed",
            Self::ResourceConflict => "The operation would create a conflict",
            Self::BusinessRuleViolation => "The operation violates business rules",
            Self::DependencyExists => "Cannot complete operation due to existing dependencies",

            Self::ExternalServiceError => "Error communicating with external service",
            Self::ExternalServiceTimeout => "External service request timed out",
            Self::ExternalServiceUnavailable => "External service is unavailable",

            Self::FileUploadError => "Failed to upload file",
            Self::FileNotFound => "File not found",
            Self::FileTooLarge => "File size exceeds maximum allowed limit",
            Self::InvalidFileType => "File type is not supported",
            Self::StorageError => "Error storing file in the storage service",
            Self::FileDeletionError => "Failed to delete file from storage",
            Self::AssetMetadataError => "Error processing asset metadata",

            Self::EmailSendingError => "Failed to send email",
            Self::InvalidEmailFormat => "Invalid email format",
            Self::EmailDeliveryError => "Email delivery failed",

            Self::PostNotFound => "Post not found",
            Self::InvalidPostStatus => "Invalid post status",
            Self::PostAlreadyPublished => "Post is already published",
            Self::SlugAlreadyExists => "A post with this slug already exists",

            Self::CategoryNotFound => "Category not found",
            Self::CategoryInUse => "Category is in use and cannot be deleted",
            Self::InvalidCategoryParent => "Invalid parent category",

            Self::TagNotFound => "Tag not found",
            Self::TagAlreadyExists => "Tag already exists",
        }
    }

    /// Returns the HTTP status code that best represents this error
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::PasswordResetRequired => StatusCode::FORBIDDEN,
            Self::AccountLocked => StatusCode::FORBIDDEN,
            Self::TooManyAttempts => StatusCode::TOO_MANY_REQUESTS,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::EmailVerificationRequired => StatusCode::FORBIDDEN,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,

            Self::InvalidInput => StatusCode::BAD_REQUEST,
            Self::MissingRequiredField => StatusCode::BAD_REQUEST,
            Self::InvalidFormat => StatusCode::BAD_REQUEST,
            Self::InvalidLength => StatusCode::BAD_REQUEST,
            Self::InvalidValue => StatusCode::BAD_REQUEST,
            Self::ValidationError => StatusCode::BAD_REQUEST,

            Self::DatabaseConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RecordNotFound => StatusCode::NOT_FOUND,
            Self::DuplicateEntry => StatusCode::CONFLICT,
            Self::QueryError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TransactionError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RelationshipError => StatusCode::BAD_REQUEST,
            Self::IntegrityError => StatusCode::CONFLICT,

            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::ConfigurationError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::OperationNotAllowed => StatusCode::FORBIDDEN,
            Self::ResourceConflict => StatusCode::CONFLICT,
            Self::BusinessRuleViolation => StatusCode::UNPROCESSABLE_ENTITY,
            Self::DependencyExists => StatusCode::CONFLICT,

            Self::ExternalServiceError => StatusCode::BAD_GATEWAY,
            Self::ExternalServiceTimeout => StatusCode::GATEWAY_TIMEOUT,
            Self::ExternalServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,

            Self::FileUploadError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FileNotFound => StatusCode::NOT_FOUND,
            Self::FileTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::InvalidFileType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::StorageError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FileDeletionError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AssetMetadataError => StatusCode::BAD_REQUEST,

            Self::EmailSendingError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidEmailFormat => StatusCode::BAD_REQUEST,
            Self::EmailDeliveryError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::PostNotFound => StatusCode::NOT_FOUND,
            Self::InvalidPostStatus => StatusCode::BAD_REQUEST,
            Self::PostAlreadyPublished => StatusCode::CONFLICT,
            Self::SlugAlreadyExists => StatusCode::CONFLICT,

            Self::CategoryNotFound => StatusCode::NOT_FOUND,
            Self::CategoryInUse => StatusCode::CONFLICT,
            Self::InvalidCategoryParent => StatusCode::BAD_REQUEST,

            Self::TagNotFound => StatusCode::NOT_FOUND,
            Self::TagAlreadyExists => StatusCode::CONFLICT,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json_string =
            serde_json::to_string(self).unwrap_or_else(|_| "\"UNKNOWN_ERROR\"".to_string());
        // Remove the quotes that serde_json adds
        write!(f, "{}", json_string.trim_matches('"'))
    }
}
