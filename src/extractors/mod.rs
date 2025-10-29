//! Custom extractors for the application
//!
//! This module provides specialized extractors that simplify
//! request handling in controllers.

pub mod validated;
pub mod multipart;

pub use validated::*;
pub use multipart::*;
