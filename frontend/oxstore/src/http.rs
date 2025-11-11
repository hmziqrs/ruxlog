use crate::error::TransportErrorKind;
use serde::de::DeserializeOwned;

/// Best-effort offline detection (wasm only). Returns false on non-wasm targets.
pub fn is_offline() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .map(|w| w.navigator())
            .map(|n| !n.on_line())
            .unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Heuristically classify a transport error and produce a user-facing message.
/// This is a default implementation that consuming libraries can override.
pub fn classify_transport_error<E: std::fmt::Debug>(e: &E) -> (TransportErrorKind, String) {
    if is_offline() {
        return (
            TransportErrorKind::Offline,
            "You appear to be offline".to_string(),
        );
    }

    // Default classification for generic error types
    (TransportErrorKind::Unknown, format!("{:?}", e))
}

/// Generic HTTP response trait for state management abstractions
pub trait HttpResponse {
    /// Get the HTTP status code
    fn status(&self) -> u16;

    /// Get response body as text
    async fn text(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// Parse response body as JSON
    async fn json<T: DeserializeOwned>(&self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

/// Generic HTTP error trait for error classification
pub trait HttpError: std::error::Error {
    /// Classify the transport error for user-friendly messages
    fn classify_transport_error(&self) -> (TransportErrorKind, String);
}