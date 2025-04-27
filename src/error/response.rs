//! Standardized error response structures
//!
//! This module defines the standard error response format for the API.

use std::fmt;
use serde::{Serialize, Deserialize};
use axum::{response::IntoResponse, Json};
use super::codes::ErrorCode;

/// Standard error response structure for API responses
///
/// This structure provides a consistent error format that includes:
/// - An error code (which can be used for translation)
/// - A human-readable message (which may be localized on the server if Accept-Language is used)
/// - Optional detailed information for developers (only in development mode)
/// - Optional additional fields for specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// The error type - this will serialize to strings like "AUTH_001"
    #[serde(rename = "type")]
    pub code: ErrorCode,
    
    #[cfg(debug_assertions)]
    /// Human-readable error message (only in development)
    pub message: String,

    #[cfg(not(debug_assertions))]
    #[serde(skip)]
    /// Human-readable error message (skipped in production)
    pub message: String,
    
    /// HTTP status code
    pub status: u16,
    
    /// Optional detailed description (only included in development mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(debug_assertions)]
    pub details: Option<String>,
    
    /// Optional additional information specific to the error type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    
    /// Request ID for tracing (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response with the given error code
    ///
    /// Uses the default message for the error code.
    pub fn new(code: ErrorCode) -> Self {
        let status = code.status_code();
        Self {
            message: code.default_message().to_string(),
            code,
            status: status.as_u16(),
            details: None,
            context: None,
            request_id: None,
        }
    }
    
    /// Add a custom message to the error response
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
    
    /// Add detailed information (only included in development mode)
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        // In production, this should check an environment variable
        // and only include details in development mode
        #[cfg(debug_assertions)]
        {
            self.details = Some(details.into());
        }
        self
    }
    
    /// Add context information to the error
    pub fn with_context(mut self, context: impl Serialize) -> Self {
        match serde_json::to_value(context) {
            Ok(value) => self.context = Some(value),
            Err(err) => {
                // Log the error but don't fail
                eprintln!("Failed to serialize error context: {}", err);
            }
        }
        self
    }
    
    /// Add a request ID for tracing
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        
        // Get the status code
        let status = StatusCode::from_u16(self.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        // Log the error if it's a server error (5xx)
        if status.is_server_error() {
            eprintln!("Server error {}: {}", self.code, self.message);
            if let Some(details) = &self.details {
                eprintln!("  Details: {}", details);
            }
        }
        
        // Convert to a response
        (status, Json(self)).into_response()
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Extension trait to convert various error types to ErrorResponse
pub trait IntoErrorResponse {
    /// Convert this error into a standard ErrorResponse
    fn into_error_response(self) -> ErrorResponse;
}