//! Custom extractors for the application
//!
//! This module provides specialized extractors that simplify
//! request handling in controllers.

pub mod multipart;
pub mod validated;

pub use multipart::*;
pub use validated::*;
