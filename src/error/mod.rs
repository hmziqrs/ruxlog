//! Global error handling module for the application
//!
//! This module provides standardized error codes, error response structures,
//! and utilities for error handling throughout the application.

pub mod response;
pub mod codes;
pub mod auth;

pub use response::ErrorResponse;
pub use codes::ErrorCode;
pub use response::IntoErrorResponse;