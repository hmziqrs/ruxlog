#![allow(clippy::module_inception)]

// Utilities root module.
// Re-export color-related helpers so callers can `use crate::utils::*` or `utils::...`.

pub mod color;
pub use color::*;
