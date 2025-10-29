//! Error handling for validation and JSON parsing errors

use crate::error::{ErrorCode, ErrorResponse, IntoErrorResponse};
use axum::extract::rejection::{FormRejection, JsonRejection, QueryRejection};
use axum::extract::multipart::MultipartRejection;
use serde_json::Value;
use std::error::Error as StdError;
use validator::ValidationErrors;

/// Standardized handling for JSON validation errors
impl IntoErrorResponse for JsonRejection {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            JsonRejection::JsonDataError(err) => {
                if let Some(validation_err) = err
                    .source()
                    .and_then(|source| source.downcast_ref::<ValidationErrors>())
                {
                    let error_json = serde_json::to_value(validation_err).unwrap_or(Value::Null);

                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Validation failed")
                        .with_context(error_json)
                } else {
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Invalid JSON input data")
                        .with_details(err.to_string())
                }
            }
            JsonRejection::JsonSyntaxError(err) => ErrorResponse::new(ErrorCode::InvalidFormat)
                .with_message("Invalid JSON syntax")
                .with_details(err.to_string()),
            JsonRejection::MissingJsonContentType(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Missing or invalid Content-Type header")
                    .with_details(err.to_string())
            }
            JsonRejection::BytesRejection(err) => ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Failed to extract request body")
                .with_details(err.to_string()),
            _ => ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Invalid request payload")
                .with_details(format!("Unhandled JSON rejection: {}", self)),
        }
    }
}

/// Implement From<JsonRejection> for ErrorResponse for convenience
impl From<JsonRejection> for ErrorResponse {
    fn from(err: JsonRejection) -> Self {
        err.into_error_response()
    }
}

/// Standardized handling for form validation errors
impl IntoErrorResponse for FormRejection {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            FormRejection::FailedToDeserializeForm(err) => {
                if let Some(validation_err) = err
                    .source()
                    .and_then(|source| source.downcast_ref::<ValidationErrors>())
                {
                    let error_json = serde_json::to_value(validation_err).unwrap_or(Value::Null);

                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Form validation failed")
                        .with_context(error_json)
                } else {
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Invalid form data")
                        .with_details(err.to_string())
                }
            }
            FormRejection::FailedToDeserializeFormBody(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Failed to parse form data")
                    .with_details(err.to_string())
            }
            FormRejection::InvalidFormContentType(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Missing or invalid Content-Type header for form data")
                    .with_details(err.to_string())
            }
            FormRejection::BytesRejection(err) => ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Failed to extract form body")
                .with_details(err.to_string()),
            _ => ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Invalid form data")
                .with_details(format!("Unhandled form rejection: {}", self)),
        }
    }
}

/// Implement From<FormRejection> for ErrorResponse for convenience
impl From<FormRejection> for ErrorResponse {
    fn from(err: FormRejection) -> Self {
        err.into_error_response()
    }
}

/// Standardized handling for query validation errors
impl IntoErrorResponse for QueryRejection {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            QueryRejection::FailedToDeserializeQueryString(err) => {
                if let Some(validation_err) = err
                    .source()
                    .and_then(|source| source.downcast_ref::<ValidationErrors>())
                {
                    let error_json = serde_json::to_value(validation_err).unwrap_or(Value::Null);

                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Query validation failed")
                        .with_context(error_json)
                } else {
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Invalid query parameters")
                        .with_details(err.to_string())
                }
            }
            _ => ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Invalid query parameters")
                .with_details(format!("Unhandled query rejection: {}", self)),
        }
    }
}

/// Implement From<QueryRejection> for ErrorResponse for convenience
impl From<QueryRejection> for ErrorResponse {
    fn from(err: QueryRejection) -> Self {
        err.into_error_response()
    }
}

/// Standardized handling for multipart extraction errors
impl IntoErrorResponse for MultipartRejection {
    fn into_error_response(self) -> ErrorResponse {
        // axum MultipartRejection variants can differ across versions; do a best-effort
        // categorization and always return our standardized structure with helpful messages.
        let as_string = self.to_string();

        // Common boundary errors (e.g., sending JSON to a multipart endpoint)
        if as_string.to_ascii_lowercase().contains("boundary") {
            return ErrorResponse::new(ErrorCode::InvalidFormat)
                .with_message("Invalid multipart/form-data boundary")
                .with_details(as_string)
                .with_context(serde_json::json!({
                    "hint": "Ensure the request sets 'Content-Type: multipart/form-data; boundary=...'. If you're sending JSON, use the JSON endpoint instead.",
                }));
        }

        // Content-Type issues
        if as_string
            .to_ascii_lowercase()
            .contains("content-type")
        {
            return ErrorResponse::new(ErrorCode::InvalidFormat)
                .with_message("Missing or invalid Content-Type for multipart/form-data")
                .with_details(as_string);
        }

        // Body/read errors or size-related
        if as_string.to_ascii_lowercase().contains("body")
            || as_string.to_ascii_lowercase().contains("read")
        {
            return ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Failed to read multipart body")
                .with_details(as_string);
        }

        // Fallback generic mapping
        ErrorResponse::new(ErrorCode::InvalidInput)
            .with_message("Invalid multipart/form-data payload")
            .with_details(format!("Unhandled multipart rejection: {}", self))
    }
}

/// Implement From<MultipartRejection> for ErrorResponse for convenience
impl From<MultipartRejection> for ErrorResponse {
    fn from(err: MultipartRejection) -> Self {
        err.into_error_response()
    }
}
