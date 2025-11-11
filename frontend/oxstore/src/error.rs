use serde::{Deserialize, Serialize};

/// Generic error types for state management

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TransportErrorKind {
    Offline,
    Network,
    Timeout,
    Canceled,
    Unknown,
}

impl TransportErrorKind {
    pub fn label(&self) -> &'static str {
        match self {
            TransportErrorKind::Offline => "Offline",
            TransportErrorKind::Network => "Network",
            TransportErrorKind::Timeout => "Timeout",
            TransportErrorKind::Canceled => "Canceled",
            TransportErrorKind::Unknown => "Unknown",
        }
    }

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub r#type: Option<String>,
    pub message: Option<String>,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn message(&self) -> String {
        if let Some(m) = &self.message {
            return m.clone();
        }
        let ty = self.r#type.as_deref().unwrap_or("");
        if ty.is_empty() {
            format!("Request failed (status {})", self.status)
        } else {
            format!(
                "Request failed with type {} (status {})",
                ty, self.status
            )
        }
    }
}

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
/// This is a generic implementation - consuming libraries can provide their own specific implementations.
pub fn classify_transport_error<E: std::fmt::Debug>(e: &E) -> (TransportErrorKind, String) {
    if is_offline() {
        return (
            TransportErrorKind::Offline,
            "You appear to be offline".to_string(),
        );
    }

    // Default to unknown error for generic implementations
    (TransportErrorKind::Unknown, format!("{:?}", e))
}