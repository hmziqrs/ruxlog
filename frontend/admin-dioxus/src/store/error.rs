use crate::services::http_client::HttpError;
use serde::{Deserialize, Serialize};

/// Unified error carried by StateFrame
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Api(ApiError),
    Transport(TransportErrorInfo),
    Decode {
        label: String,
        error: String,
        raw: Option<String>,
    },
    Other {
        message: String,
    },
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            AppError::Api(api) => api.message(),
            AppError::Transport(t) => match t.kind {
                TransportErrorKind::Offline => "You appear to be offline".to_string(),
                TransportErrorKind::Network => t
                    .message
                    .clone()
                    .unwrap_or_else(|| "API server is unreachable".to_string()),
                TransportErrorKind::Timeout => t
                    .message
                    .clone()
                    .unwrap_or_else(|| "Request timed out".to_string()),
                TransportErrorKind::Canceled => t
                    .message
                    .clone()
                    .unwrap_or_else(|| "Request canceled".to_string()),
                TransportErrorKind::Unknown => t
                    .message
                    .clone()
                    .unwrap_or_else(|| "Network error".to_string()),
            },
            AppError::Decode { label, error, .. } => {
                format!("Unexpected response format for '{}': {}", label, error)
            }
            AppError::Other { message } => message.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Error type string coming from server under the JSON key "type" (e.g., "AUTH_001")
    pub r#type: Option<String>,
    /// Human-readable message (may be omitted in production builds of the server)
    pub message: Option<String>,
    /// HTTP status code echoed by the backend
    pub status: u16,
    /// Optional detailed description (present only in development on the server)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Optional additional structured context for the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Optional Retry-After seconds if the request is rate-limited
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
    /// Optional request id for tracing/correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    /// Return a user-facing message for this error, with sensible fallbacks.
    pub fn message(&self) -> String {
        if let Some(m) = &self.message {
            return m.clone();
        }
        let ty = self.r#type.as_deref().unwrap_or("");
        if ty.is_empty() {
            format!("Request failed (status {})", self.status)
        } else {
            format!("Request failed with type {} (status {})", ty, self.status)
        }
    }
}

/// Transport-layer error information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportErrorKind {
    Offline,
    Network,
    Timeout,
    Canceled,
    Unknown,
}

impl TransportErrorKind {
    /// Human-readable label used by UI surfaces.
    pub fn label(&self) -> &'static str {
        match self {
            TransportErrorKind::Offline => "Offline",
            TransportErrorKind::Network => "Network",
            TransportErrorKind::Timeout => "Timeout",
            TransportErrorKind::Canceled => "Canceled",
            TransportErrorKind::Unknown => "Unknown",
        }
    }

    /// Suggested next-step hint for this error kind.
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            TransportErrorKind::Offline => Some("Reconnect to the internet and try again."),
            TransportErrorKind::Network => {
                Some("Ensure the API server is running and proxy/CORS settings allow access.")
            }
            TransportErrorKind::Timeout => {
                Some("The request timed out. Retry or inspect backend latency.")
            }
            TransportErrorKind::Canceled => Some("The browser canceled this request."),
            TransportErrorKind::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportErrorInfo {
    pub kind: TransportErrorKind,
    pub message: Option<String>,
}

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
pub fn classify_transport_error(e: &HttpError) -> (TransportErrorKind, String) {
    if is_offline() {
        return (
            TransportErrorKind::Offline,
            "You appear to be offline".to_string(),
        );
    }

    match e {
        HttpError::SerdeError(se) => {
            return (
                TransportErrorKind::Unknown,
                format!("JSON serialization error: {}", se),
            );
        }
        HttpError::JsError(js) => {
            let name = js.name.to_lowercase();
            let msg = js.message.to_lowercase();

            if name.contains("abort") || msg.contains("abort") || msg.contains("canceled") {
                return (TransportErrorKind::Canceled, "Request canceled".to_string());
            }

            if msg.contains("timeout") || msg.contains("timed out") || msg.contains("etimedout") {
                return (TransportErrorKind::Timeout, "Request timed out".to_string());
            }

            if msg.contains("dns") || msg.contains("resolve") || msg.contains("name not resolved") {
                return (
                    TransportErrorKind::Network,
                    "Could not resolve API host".to_string(),
                );
            }

            if msg.contains("cors") || msg.contains("blocked by cors") {
                return (
                    TransportErrorKind::Network,
                    "Request blocked by CORS configuration".to_string(),
                );
            }

            if msg.contains("failed to fetch")
                || msg.contains("networkerror")
                || msg.contains("network error")
                || name.contains("typeerror")
            {
                return (
                    TransportErrorKind::Network,
                    "API server is unreachable".to_string(),
                );
            }

            return (
                TransportErrorKind::Network,
                format!("{}: {}", js.name, js.message),
            );
        }
        HttpError::GlooError(s) => {
            let s_l = s.to_lowercase();
            if s_l.contains("timeout") {
                return (TransportErrorKind::Timeout, "Request timed out".to_string());
            }
            (TransportErrorKind::Unknown, s.clone())
        }
    }
}
