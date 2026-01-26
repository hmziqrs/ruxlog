pub mod config;
pub mod db;
pub mod error;
pub mod extractors;
pub mod middlewares;
pub mod modules;
pub mod router;
pub mod services;
pub mod state;
pub mod utils;

#[cfg(feature = "seed-system")]
pub mod tui;

pub use crate::state::AppState;
