//! Error handling for validation and JSON parsing errors

use std::error::Error as StdError;
use crate::error::{ErrorCode, ErrorResponse, IntoErrorResponse};
use axum::extract::rejection::{JsonRejection, FormRejection, QueryRejection};
use validator::ValidationErrors;
use serde_json::Value;

/// Standardized handling for JSON validation errors
impl IntoErrorResponse for JsonRejection {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            JsonRejection::JsonDataError(err) => {
                // Check if this is a validation error
                if let Some(validation_err) = err.source().and_then(|source| source.downcast_ref::<ValidationErrors>()) {
                    // Convert ValidationErrors to a JSON-serializable format
                    let error_json = serde_json::to_value(validation_err).unwrap_or(Value::Null);
                    
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Validation failed")
                        .with_context(error_json)
                }
                else {
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Invalid JSON input data")
                        .with_details(err.to_string())
                }
            },
            JsonRejection::JsonSyntaxError(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Invalid JSON syntax")
                    .with_details(err.to_string())
            },
            JsonRejection::MissingJsonContentType(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Missing or invalid Content-Type header")
                    .with_details(err.to_string())
            },
            JsonRejection::BytesRejection(err) => {
                ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("Failed to extract request body")
                    .with_details(err.to_string())
            },
            _ => {
                ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("Invalid request payload")
                    .with_details(format!("Unhandled JSON rejection: {}", self))
            }
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
                // Check if this is a validation error
                if let Some(validation_err) = err.source().and_then(|source| source.downcast_ref::<ValidationErrors>()) {
                    // Convert ValidationErrors to a JSON-serializable format
                    let error_json = serde_json::to_value(validation_err).unwrap_or(Value::Null);
                    
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Form validation failed")
                        .with_context(error_json)
                }
                else {
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Invalid form data")
                        .with_details(err.to_string())
                }
            },
            FormRejection::FailedToDeserializeFormBody(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Failed to parse form data")
                    .with_details(err.to_string())
            },
            FormRejection::InvalidFormContentType(err) => {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Missing or invalid Content-Type header for form data")
                    .with_details(err.to_string())
            },
            FormRejection::BytesRejection(err) => {
                ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("Failed to extract form body")
                    .with_details(err.to_string())
            },
            _ => {
                ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("Invalid form data")
                    .with_details(format!("Unhandled form rejection: {}", self))
            }
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
                // Check if this is a validation error
                if let Some(validation_err) = err.source().and_then(|source| source.downcast_ref::<ValidationErrors>()) {
                    // Convert ValidationErrors to a JSON-serializable format
                    let error_json = serde_json::to_value(validation_err).unwrap_or(Value::Null);
                    
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Query validation failed")
                        .with_context(error_json)
                }
                else {
                    ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("Invalid query parameters")
                        .with_details(err.to_string())
                }
            },
            _ => {
                ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("Invalid query parameters")
                    .with_details(format!("Unhandled query rejection: {}", self))
            }
        }
    }
}

/// Implement From<QueryRejection> for ErrorResponse for convenience
impl From<QueryRejection> for ErrorResponse {
    fn from(err: QueryRejection) -> Self {
        err.into_error_response()
    }
}