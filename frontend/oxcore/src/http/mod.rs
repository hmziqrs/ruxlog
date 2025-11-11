mod config;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(not(target_arch = "wasm32"))]
mod native;

// Common form data type for cross-platform compatibility
#[cfg(target_arch = "wasm32")]
pub use web_sys::FormData;

#[cfg(not(target_arch = "wasm32"))]
pub use serde_json::Value as FormData;

// Re-export config
pub use config::configure;

// Re-export platform-appropriate types
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
