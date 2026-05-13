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

#[cfg(test)]
mod tests {
    use super::*;

    // ── TransportErrorKind::label ──

    #[test]
    fn label_offline() {
        assert_eq!(TransportErrorKind::Offline.label(), "Offline");
    }

    #[test]
    fn label_network() {
        assert_eq!(TransportErrorKind::Network.label(), "Network");
    }

    #[test]
    fn label_timeout() {
        assert_eq!(TransportErrorKind::Timeout.label(), "Timeout");
    }

    #[test]
    fn label_canceled() {
        assert_eq!(TransportErrorKind::Canceled.label(), "Canceled");
    }

    #[test]
    fn label_unknown() {
        assert_eq!(TransportErrorKind::Unknown.label(), "Unknown");
    }

    // ── TransportErrorKind::hint ──

    #[test]
    fn hint_offline() {
        assert_eq!(
            TransportErrorKind::Offline.hint(),
            Some("Reconnect to the internet and try again.")
        );
    }

    #[test]
    fn hint_network() {
        assert_eq!(
            TransportErrorKind::Network.hint(),
            Some("Ensure the API server is running and proxy/CORS settings allow access.")
        );
    }

    #[test]
    fn hint_timeout() {
        assert_eq!(
            TransportErrorKind::Timeout.hint(),
            Some("The request timed out. Retry or inspect backend latency.")
        );
    }

    #[test]
    fn hint_canceled() {
        assert_eq!(
            TransportErrorKind::Canceled.hint(),
            Some("The browser canceled this request.")
        );
    }

    #[test]
    fn hint_unknown_is_none() {
        assert_eq!(TransportErrorKind::Unknown.hint(), None);
    }

    // ── ApiError::message ──

    #[test]
    fn api_error_message_with_both_type_and_message() {
        let err = ApiError {
            r#type: Some("Validation".into()),
            message: Some("Name is required".into()),
            status: 422,
            details: None,
            context: None,
            retry_after: None,
            request_id: None,
        };
        assert_eq!(err.message(), "Name is required");
    }

    #[test]
    fn api_error_message_only() {
        let err = ApiError {
            r#type: None,
            message: Some("Something broke".into()),
            status: 500,
            details: None,
            context: None,
            retry_after: None,
            request_id: None,
        };
        assert_eq!(err.message(), "Something broke");
    }

    #[test]
    fn api_error_type_only() {
        let err = ApiError {
            r#type: Some("NotFound".into()),
            message: None,
            status: 404,
            details: None,
            context: None,
            retry_after: None,
            request_id: None,
        };
        assert_eq!(err.message(), "Request failed with type NotFound (status 404)");
    }

    #[test]
    fn api_error_no_type_no_message() {
        let err = ApiError {
            r#type: None,
            message: None,
            status: 503,
            details: None,
            context: None,
            retry_after: None,
            request_id: None,
        };
        assert_eq!(err.message(), "Request failed (status 503)");
    }

    #[test]
    fn api_error_empty_type_no_message() {
        let err = ApiError {
            r#type: Some(String::new()),
            message: None,
            status: 500,
            details: None,
            context: None,
            retry_after: None,
            request_id: None,
        };
        assert_eq!(err.message(), "Request failed (status 500)");
    }

    // ── AppError::message ──

    #[test]
    fn app_error_api() {
        let api = ApiError {
            r#type: Some("Auth".into()),
            message: Some("Bad token".into()),
            status: 401,
            details: None,
            context: None,
            retry_after: None,
            request_id: None,
        };
        let app = AppError::Api(api);
        assert_eq!(app.message(), "Bad token");
    }

    #[test]
    fn app_error_transport_offline() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Offline,
            message: None,
        });
        assert_eq!(app.message(), "You appear to be offline");
    }

    #[test]
    fn app_error_transport_network() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Network,
            message: Some("connection refused".into()),
        });
        assert_eq!(app.message(), "connection refused");
    }

    #[test]
    fn app_error_transport_network_default_message() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Network,
            message: None,
        });
        assert_eq!(app.message(), "API server is unreachable");
    }

    #[test]
    fn app_error_transport_timeout_with_message() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Timeout,
            message: Some("took 30s".into()),
        });
        assert_eq!(app.message(), "took 30s");
    }

    #[test]
    fn app_error_transport_timeout_default_message() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Timeout,
            message: None,
        });
        assert_eq!(app.message(), "Request timed out");
    }

    #[test]
    fn app_error_transport_canceled() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Canceled,
            message: None,
        });
        assert_eq!(app.message(), "Request canceled");
    }

    #[test]
    fn app_error_transport_unknown() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Unknown,
            message: Some("weird error".into()),
        });
        assert_eq!(app.message(), "weird error");
    }

    #[test]
    fn app_error_transport_unknown_default() {
        let app = AppError::Transport(TransportErrorInfo {
            kind: TransportErrorKind::Unknown,
            message: None,
        });
        assert_eq!(app.message(), "Network error");
    }

    #[test]
    fn app_error_decode() {
        let app = AppError::Decode {
            label: "user_list".into(),
            error: "missing field `id`".into(),
            raw: None,
        };
        assert_eq!(
            app.message(),
            "Unexpected response format for 'user_list': missing field `id`"
        );
    }

    #[test]
    fn app_error_other() {
        let app = AppError::Other {
            message: "generic issue".into(),
        };
        assert_eq!(app.message(), "generic issue");
    }

    // ── classify_transport_error ──

    #[test]
    fn classify_transport_error_non_wasm() {
        let err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let (kind, msg) = classify_transport_error(&err);
        assert_eq!(kind, TransportErrorKind::Unknown);
        assert!(msg.contains("refused"));
    }

    // ── is_offline on non-wasm ──

    #[test]
    fn is_offline_non_wasm_is_false() {
        assert!(!is_offline());
    }
}