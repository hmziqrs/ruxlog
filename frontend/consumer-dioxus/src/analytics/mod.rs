//! Firebase Analytics integration for Dioxus
//!
//! This module provides Firebase Analytics tracking capabilities for the application.
//! Only available on wasm32 target and when the `analytics` feature is enabled.
//!
//! # Features
//! - Page view tracking
//! - User engagement tracking (likes, shares, comments)
//! - Navigation tracking (category, tag, link clicks)
//! - Scroll depth tracking
//! - Time-on-page tracking
//! - Outbound link tracking
//!
//! # Usage
//!
//! Initialize Firebase in your main component:
//! ```rust
//! #[cfg(all(target_arch = "wasm32", feature = "analytics"))]
//! analytics::initialize();
//! ```
//!
//! Track events:
//! ```rust
//! use analytics::tracker;
//!
//! tracker::track_page_view("/blog", "Blog");
//! tracker::track_post_view("post-123", "My Post", Some("tech"));
//! ```
//!
//! Use hooks in components:
//! ```rust
//! use analytics::hooks::{use_page_timer, use_scroll_depth};
//!
//! fn MyComponent() -> Element {
//!     use_page_timer("/blog/my-post");
//!     use_scroll_depth("/blog/my-post");
//!     // ...
//! }
//! ```

#[cfg(feature = "analytics")]
pub mod bindings;

#[cfg(feature = "analytics")]
pub mod tracker;

#[cfg(feature = "analytics")]
pub mod hooks;

#[cfg(feature = "analytics")]
pub use tracker::*;

#[cfg(feature = "analytics")]
pub use hooks::*;

/// Initialize Firebase Analytics
///
/// This should be called once when the app starts, typically in the main App component.
/// Requires Firebase configuration environment variables to be set at compile time.
///
/// # Returns
/// `true` if initialization was successful, `false` otherwise
///
/// # Example
/// ```rust
/// #[cfg(all(target_arch = "wasm32", feature = "analytics"))]
/// if analytics::initialize() {
///     log::info!("Firebase Analytics initialized");
/// } else {
///     log::warn!("Failed to initialize Firebase Analytics");
/// }
/// ```
#[cfg(all(target_arch = "wasm32", feature = "analytics"))]
pub fn initialize() -> bool {
    use crate::env;

    let success = bindings::init_firebase(
        env::FIREBASE_API_KEY,
        env::FIREBASE_AUTH_DOMAIN,
        env::FIREBASE_PROJECT_ID,
        env::FIREBASE_STORAGE_BUCKET,
        env::FIREBASE_MESSAGING_SENDER_ID,
        env::FIREBASE_APP_ID,
        env::FIREBASE_MEASUREMENT_ID,
    );

    if success {
        log::info!("Firebase Analytics initialized successfully");
    } else {
        log::error!("Failed to initialize Firebase Analytics");
    }

    success
}

/// Check if analytics is available and enabled
#[cfg(all(target_arch = "wasm32", feature = "analytics"))]
pub fn is_available() -> bool {
    bindings::is_analytics_available()
}

// No-op functions for non-wasm or when analytics feature is disabled
#[cfg(not(all(target_arch = "wasm32", feature = "analytics")))]
pub fn initialize() -> bool {
    false
}

#[cfg(not(all(target_arch = "wasm32", feature = "analytics")))]
pub fn is_available() -> bool {
    false
}
