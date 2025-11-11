//! Global error handling module for the application
//!
//! This module provides standardized error codes, error response structures,
//! and utilities for error handling throughout the application.

pub mod auth;
pub mod codes;
pub mod database;
pub mod login;
pub mod response;
pub mod validation;

pub use codes::ErrorCode;
pub use database::{DbResult, DbResultExt};
pub use response::ErrorResponse;
pub use response::IntoErrorResponse;
