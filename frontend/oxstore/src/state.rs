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
        self.error = Some(AppError::Transport(crate::error::TransportErrorInfo { kind, message }));
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