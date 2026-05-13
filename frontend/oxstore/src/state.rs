use crate::error::{AppError, TransportErrorKind};
use serde_json;

/// Status of a state frame for tracking operation lifecycle
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StateFrameStatus {
    Init,
    Loading,
    Success,
    Failed,
}

/// Alternative status enum for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum StateStatus {
    Idle,
    Loading,
    Error(String),
    Loaded,
}

/// Generic state frame that holds data, metadata, status, and error information
#[derive(Debug, Clone, PartialEq)]
pub struct StateFrame<D: Clone = (), M: Clone = ()> {
    pub status: StateFrameStatus,
    pub data: Option<D>,
    pub meta: Option<M>,
    pub error: Option<AppError>,
}

impl<D: Clone, Q: Clone> Default for StateFrame<D, Q> {
    fn default() -> Self {
        Self {
            status: StateFrameStatus::Init,
            data: None,
            meta: None,
            error: None,
        }
    }
}

impl<D: Clone, Q: Clone> StateFrame<D, Q> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_loading() -> Self {
        Self {
            status: StateFrameStatus::Loading,
            data: None,
            meta: None,
            error: None,
        }
    }

    pub fn new_with_data(data: Option<D>) -> Self {
        Self {
            status: StateFrameStatus::Success,
            data,
            meta: None,
            error: None,
        }
    }

    pub fn is_init(&self) -> bool {
        self.status == StateFrameStatus::Init
    }

    pub fn is_loading(&self) -> bool {
        self.status == StateFrameStatus::Loading
    }

    pub fn is_success(&self) -> bool {
        self.status == StateFrameStatus::Success
    }

    pub fn is_failed(&self) -> bool {
        self.status == StateFrameStatus::Failed
    }

    pub fn set_loading(&mut self) {
        self.status = StateFrameStatus::Loading;
        self.error = None;
    }

    pub fn set_loading_meta(&mut self, meta: Option<Q>) {
        self.status = StateFrameStatus::Loading;
        self.meta = meta;
        self.error = None;
    }

    pub fn set_success(&mut self, data: Option<D>) {
        self.status = StateFrameStatus::Success;
        self.data = data;
        self.error = None;
    }

    pub fn set_failed(&mut self, message: String) {
        self.status = StateFrameStatus::Failed;
        self.error = Some(AppError::Other { message });
    }

    pub fn set_meta(&mut self, meta: Option<Q>) {
        self.meta = meta;
    }

    pub fn set_api_error(&mut self, status: u16, body: String) {
        self.status = StateFrameStatus::Failed;

        match serde_json::from_str::<crate::error::ApiError>(&body) {
            Ok(mut api_error) => {
                if api_error.message.is_none() {
                    let ty = api_error.r#type.clone().unwrap_or_default();
                    api_error.message = Some(if ty.is_empty() {
                        format!("Request failed (status {})", api_error.status)
                    } else {
                        format!(
                            "Request failed with type {} (status {})",
                            ty, api_error.status
                        )
                    });
                }

                self.error = Some(AppError::Api(api_error));
            }
            Err(e) => {
                self.error = Some(AppError::Decode {
                    label: "api_error".to_string(),
                    error: format!("Failed to parse API error (status {}): {}", status, e),
                    raw: if body.is_empty() { None } else { Some(body) },
                });
            }
        }
    }

    pub fn set_transport_error(&mut self, kind: TransportErrorKind, message: Option<String>) {
        self.status = StateFrameStatus::Failed;
        self.error = Some(AppError::Transport(crate::error::TransportErrorInfo {
            kind,
            message,
        }));
    }

    pub fn set_decode_error(
        &mut self,
        label: impl Into<String>,
        err: impl Into<String>,
        raw: Option<String>,
    ) {
        self.status = StateFrameStatus::Failed;
        let label_s = label.into();
        let err_s = err.into();
        self.error = Some(AppError::Decode {
            label: label_s,
            error: err_s,
            raw,
        });
    }

    /// Convenience: unified error message if any
    pub fn error_message(&self) -> Option<String> {
        self.error.as_ref().map(|f| f.message())
    }

    pub fn error_type(&self) -> Option<&str> {
        match &self.error {
            Some(AppError::Api(api)) => api.r#type.as_deref(),
            _ => None,
        }
    }

    pub fn error_status(&self) -> Option<u16> {
        match &self.error {
            Some(AppError::Api(api)) => Some(api.status),
            _ => None,
        }
    }

    pub fn error_details(&self) -> Option<&str> {
        match &self.error {
            Some(AppError::Api(api)) => api.details.as_deref(),
            _ => None,
        }
    }

    pub fn error_or_message(&self, fallback: impl Into<String>) -> AppError {
        self.error.clone().unwrap_or_else(|| AppError::Other {
            message: fallback.into(),
        })
    }

    pub fn is_offline(&self) -> bool {
        matches!(
            self.transport_error_kind(),
            Some(TransportErrorKind::Offline)
        )
    }

    pub fn transport_error_kind(&self) -> Option<TransportErrorKind> {
        match &self.error {
            Some(AppError::Transport(t)) => Some(t.kind),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Constructors ──

    #[test]
    fn new_is_init() {
        let frame: StateFrame<String> = StateFrame::new();
        assert_eq!(frame.status, StateFrameStatus::Init);
        assert!(frame.data.is_none());
        assert!(frame.meta.is_none());
        assert!(frame.error.is_none());
    }

    #[test]
    fn new_with_loading() {
        let frame: StateFrame<String> = StateFrame::new_with_loading();
        assert_eq!(frame.status, StateFrameStatus::Loading);
        assert!(frame.data.is_none());
        assert!(frame.error.is_none());
    }

    #[test]
    fn new_with_data_some() {
        let frame: StateFrame<String> = StateFrame::new_with_data(Some("hello".to_string()));
        assert_eq!(frame.status, StateFrameStatus::Success);
        assert_eq!(frame.data, Some("hello".to_string()));
        assert!(frame.error.is_none());
    }

    #[test]
    fn new_with_data_none() {
        let frame: StateFrame<Option<String>> = StateFrame::new_with_data(None);
        assert_eq!(frame.status, StateFrameStatus::Success);
        assert!(frame.data.is_none());
    }

    // ── Status checks ──

    #[test]
    fn is_init_on_new() {
        let frame: StateFrame<()> = StateFrame::new();
        assert!(frame.is_init());
        assert!(!frame.is_loading());
        assert!(!frame.is_success());
        assert!(!frame.is_failed());
    }

    #[test]
    fn is_loading() {
        let frame: StateFrame<()> = StateFrame::new_with_loading();
        assert!(frame.is_loading());
        assert!(!frame.is_init());
    }

    #[test]
    fn is_success() {
        let frame: StateFrame<i32> = StateFrame::new_with_data(Some(42));
        assert!(frame.is_success());
        assert!(!frame.is_loading());
    }

    #[test]
    fn is_failed() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_failed("oops".into());
        assert!(frame.is_failed());
        assert!(!frame.is_success());
    }

    // ── State transitions ──

    #[test]
    fn set_loading_clears_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_failed("bad".into());
        assert!(frame.is_failed());

        frame.set_loading();
        assert!(frame.is_loading());
        assert!(frame.error.is_none());
    }

    #[test]
    fn set_success_clears_error() {
        let mut frame: StateFrame<i32> = StateFrame::new();
        frame.set_failed("bad".into());
        frame.set_success(Some(10));
        assert!(frame.is_success());
        assert_eq!(frame.data, Some(10));
        assert!(frame.error.is_none());
    }

    #[test]
    fn set_failed() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_failed("something went wrong".into());
        assert!(frame.is_failed());
        assert_eq!(
            frame.error_message(),
            Some("something went wrong".to_string())
        );
    }

    // ── set_api_error with valid JSON ──

    #[test]
    fn set_api_error_valid_json_with_message() {
        let mut frame: StateFrame<()> = StateFrame::new();
        let body = r#"{"type":"Validation","message":"Name is required","status":422}"#;
        frame.set_api_error(422, body.to_string());
        assert!(frame.is_failed());
        assert_eq!(
            frame.error_message(),
            Some("Name is required".to_string())
        );
        assert_eq!(frame.error_type(), Some("Validation"));
        assert_eq!(frame.error_status(), Some(422));
    }

    #[test]
    fn set_api_error_valid_json_without_message() {
        let mut frame: StateFrame<()> = StateFrame::new();
        let body = r#"{"type":"NotFound","status":404}"#;
        frame.set_api_error(404, body.to_string());
        assert!(frame.is_failed());
        assert_eq!(
            frame.error_message(),
            Some("Request failed with type NotFound (status 404)".to_string())
        );
        assert_eq!(frame.error_type(), Some("NotFound"));
    }

    #[test]
    fn set_api_error_valid_json_no_type_no_message() {
        let mut frame: StateFrame<()> = StateFrame::new();
        let body = r#"{"status":500}"#;
        frame.set_api_error(500, body.to_string());
        assert!(frame.is_failed());
        assert_eq!(
            frame.error_message(),
            Some("Request failed (status 500)".to_string())
        );
        assert_eq!(frame.error_type(), None);
    }

    // ── set_api_error with invalid JSON (decode fallback) ──

    #[test]
    fn set_api_error_invalid_json_falls_back_to_decode() {
        let mut frame: StateFrame<()> = StateFrame::new();
        let body = "this is not json";
        frame.set_api_error(502, body.to_string());
        assert!(frame.is_failed());
        let msg = frame.error_message().unwrap();
        assert!(msg.contains("api_error"));
        assert!(msg.contains("Failed to parse API error"));
        // Decode error: no type, no status accessor
        assert_eq!(frame.error_type(), None);
        assert_eq!(frame.error_status(), None);
    }

    #[test]
    fn set_api_error_empty_body_falls_back_to_decode() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_api_error(503, String::new());
        assert!(frame.is_failed());
        let msg = frame.error_message().unwrap();
        assert!(msg.contains("api_error"));
    }

    // ── Error accessors ──

    #[test]
    fn error_message_none_on_init() {
        let frame: StateFrame<()> = StateFrame::new();
        assert!(frame.error_message().is_none());
    }

    #[test]
    fn error_type_none_on_non_api_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_failed("oops".into());
        assert_eq!(frame.error_type(), None);
    }

    #[test]
    fn error_status_none_on_non_api_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_failed("oops".into());
        assert_eq!(frame.error_status(), None);
    }

    #[test]
    fn error_type_and_status_on_api_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        let body = r#"{"type":"Auth","message":"Bad token","status":401}"#;
        frame.set_api_error(401, body.to_string());
        assert_eq!(frame.error_type(), Some("Auth"));
        assert_eq!(frame.error_status(), Some(401));
    }

    // ── Transport error helpers ──

    #[test]
    fn set_transport_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_transport_error(TransportErrorKind::Offline, None);
        assert!(frame.is_failed());
        assert_eq!(frame.transport_error_kind(), Some(TransportErrorKind::Offline));
        assert!(frame.is_offline());
    }

    #[test]
    fn is_offline_false_on_api_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_api_error(500, r#"{"status":500}"#.to_string());
        assert!(!frame.is_offline());
    }

    // ── set_decode_error ──

    #[test]
    fn set_decode_error() {
        let mut frame: StateFrame<()> = StateFrame::new();
        frame.set_decode_error("user_list", "missing field `id`", Some("raw data".into()));
        assert!(frame.is_failed());
        let msg = frame.error_message().unwrap();
        assert!(msg.contains("user_list"));
        assert!(msg.contains("missing field `id`"));
    }

    // ── Meta operations ──

    #[test]
    fn set_loading_meta() {
        let mut frame: StateFrame<i32, String> = StateFrame::new();
        frame.set_loading_meta(Some("searching".into()));
        assert!(frame.is_loading());
        assert_eq!(frame.meta, Some("searching".into()));
    }

    #[test]
    fn set_meta() {
        let mut frame: StateFrame<i32, u64> = StateFrame::new();
        frame.set_meta(Some(42));
        assert_eq!(frame.meta, Some(42));
    }
}
