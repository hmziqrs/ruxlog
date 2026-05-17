use serde::{Deserialize, Serialize};
use std::fmt;

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

    #[serde(rename = "NWS_001")]
    SubscriberNotFound,
}

impl ErrorCode {
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

            Self::SubscriberNotFound => "Newsletter subscriber not found",
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            Self::InvalidCredentials => 401,
            Self::SessionExpired => 401,
            Self::Unauthorized => 403,
            Self::PasswordResetRequired => 403,
            Self::AccountLocked => 403,
            Self::TooManyAttempts => 429,
            Self::UserNotFound => 404,
            Self::EmailVerificationRequired => 403,
            Self::InvalidToken => 401,

            Self::InvalidInput => 400,
            Self::MissingRequiredField => 400,
            Self::InvalidFormat => 400,
            Self::InvalidLength => 400,
            Self::InvalidValue => 400,
            Self::ValidationError => 400,

            Self::DatabaseConnectionError => 500,
            Self::RecordNotFound => 404,
            Self::DuplicateEntry => 409,
            Self::QueryError => 500,
            Self::TransactionError => 500,
            Self::RelationshipError => 400,
            Self::IntegrityError => 409,

            Self::InternalServerError => 500,
            Self::ServiceUnavailable => 503,
            Self::Timeout => 504,
            Self::RateLimited => 429,
            Self::ConfigurationError => 500,

            Self::OperationNotAllowed => 403,
            Self::ResourceConflict => 409,
            Self::BusinessRuleViolation => 422,
            Self::DependencyExists => 409,

            Self::ExternalServiceError => 502,
            Self::ExternalServiceTimeout => 504,
            Self::ExternalServiceUnavailable => 503,

            Self::FileUploadError => 500,
            Self::FileNotFound => 404,
            Self::FileTooLarge => 413,
            Self::InvalidFileType => 415,
            Self::StorageError => 500,
            Self::FileDeletionError => 500,
            Self::AssetMetadataError => 400,

            Self::EmailSendingError => 500,
            Self::InvalidEmailFormat => 400,
            Self::EmailDeliveryError => 500,

            Self::PostNotFound => 404,
            Self::InvalidPostStatus => 400,
            Self::PostAlreadyPublished => 409,
            Self::SlugAlreadyExists => 409,

            Self::CategoryNotFound => 404,
            Self::CategoryInUse => 409,
            Self::InvalidCategoryParent => 400,

            Self::TagNotFound => 404,
            Self::TagAlreadyExists => 409,

            Self::SubscriberNotFound => 404,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json_string =
            serde_json::to_string(self).unwrap_or_else(|_| "\"UNKNOWN_ERROR\"".to_string());
        write!(f, "{}", json_string.trim_matches('"'))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            message: code.default_message().to_string(),
            details: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codes_have_non_empty_default_messages() {
        let codes = [
            ErrorCode::InvalidCredentials,
            ErrorCode::UserNotFound,
            ErrorCode::SessionExpired,
            ErrorCode::Unauthorized,
            ErrorCode::DatabaseConnectionError,
            ErrorCode::RecordNotFound,
            ErrorCode::DuplicateEntry,
            ErrorCode::InternalServerError,
            ErrorCode::RateLimited,
            ErrorCode::FileTooLarge,
            ErrorCode::PostNotFound,
            ErrorCode::CategoryNotFound,
            ErrorCode::TagNotFound,
        ];
        for code in &codes {
            assert!(
                !code.default_message().is_empty(),
                "{:?} has empty message",
                code
            );
        }
    }

    #[test]
    fn status_code_mappings() {
        assert_eq!(ErrorCode::InvalidCredentials.status_code(), 401);
        assert_eq!(ErrorCode::Unauthorized.status_code(), 403);
        assert_eq!(ErrorCode::TooManyAttempts.status_code(), 429);
        assert_eq!(ErrorCode::RecordNotFound.status_code(), 404);
        assert_eq!(ErrorCode::DuplicateEntry.status_code(), 409);
        assert_eq!(ErrorCode::InternalServerError.status_code(), 500);
        assert_eq!(ErrorCode::FileTooLarge.status_code(), 413);
        assert_eq!(ErrorCode::ServiceUnavailable.status_code(), 503);
        assert_eq!(ErrorCode::Timeout.status_code(), 504);
        assert_eq!(ErrorCode::BusinessRuleViolation.status_code(), 422);
    }

    #[test]
    fn display_format_matches_serde_rename() {
        assert_eq!(ErrorCode::InvalidCredentials.to_string(), "AUTH_001");
        assert_eq!(ErrorCode::RecordNotFound.to_string(), "DB_002");
        assert_eq!(ErrorCode::InternalServerError.to_string(), "SRV_001");
        assert_eq!(ErrorCode::PostNotFound.to_string(), "PST_001");
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&ErrorCode::InvalidCredentials).unwrap();
        assert_eq!(json, "\"AUTH_001\"");
        let parsed: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ErrorCode::InvalidCredentials);
    }

    #[test]
    fn error_response_serialization() {
        let resp = ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Post not found")
            .with_details("id=42");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["type"], "DB_002");
        assert_eq!(json["message"], "Post not found");
        assert_eq!(json["details"], "id=42");
    }

    #[test]
    fn error_response_skips_none_details() {
        let resp = ErrorResponse::new(ErrorCode::RateLimited);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("details"));
    }
}
