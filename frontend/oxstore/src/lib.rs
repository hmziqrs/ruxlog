use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

/// Generic error types
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
            format!("Request failed with type {} (status {})", ty, self.status)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StateFrameStatus {
    Init,
    Loading,
    Success,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateStatus {
    Idle,
    Loading,
    Error(String),
    Loaded,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateFrame<D: Clone = (), M: Clone = ()> {
    pub status: StateFrameStatus,
    pub data: Option<D>,
    pub meta: Option<M>,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PaginatedList<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

impl<T> PaginatedList<T> {
    pub fn has_next_page(&self) -> bool {
        self.page * self.per_page < self.total
    }

    pub fn has_previous_page(&self) -> bool {
        self.page > 1
    }
}

impl<T> std::ops::Deref for PaginatedList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for PaginatedList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> IntoIterator for PaginatedList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T: Clone, Q: Clone> Default for StateFrame<T, Q> {
    fn default() -> Self {
        Self {
            status: StateFrameStatus::Init,
            data: None,
            meta: None,
            error: None,
        }
    }
}

// Sorting utilities

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Order::Desc
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SortParam {
    pub field: String,
    #[serde(
        default = "default_order",
        deserialize_with = "deserialize_order",
        serialize_with = "serialize_order"
    )]
    pub order: Order,
}

impl Default for SortParam {
    fn default() -> Self {
        Self {
            field: String::new(),
            order: Order::default(),
        }
    }
}

fn default_order() -> Order {
    Order::Desc
}

fn serialize_order<S>(order: &Order, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match order {
        Order::Asc => serializer.serialize_str("asc"),
        Order::Desc => serializer.serialize_str("desc"),
    }
}

fn deserialize_order<'de, D>(deserializer: D) -> Result<Order, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "asc" | "ASC" | "Asc" => Ok(Order::Asc),
        "desc" | "DESC" | "Desc" => Ok(Order::Desc),
        other => Err(serde::de::Error::custom(format!(
            "invalid order '{}', expected 'asc' or 'desc'",
            other
        ))),
    }
}

impl<T: Clone, Q: Clone> StateFrame<T, Q> {
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

    pub fn new_with_data(data: Option<T>) -> Self {
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

    pub fn set_success(&mut self, data: Option<T>) {
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

        match serde_json::from_str::<ApiError>(&body) {
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
        self.error = Some(AppError::Transport(TransportErrorInfo { kind, message }));
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

// List management traits and structures

pub trait ListQuery: Clone + Default + Serialize + for<'de> Deserialize<'de> + PartialEq {
    fn new() -> Self;

    fn page(&self) -> u64;

    fn set_page(&mut self, page: u64);

    fn search(&self) -> Option<String>;

    fn set_search(&mut self, search: Option<String>);

    fn sorts(&self) -> Option<Vec<SortParam>>;

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>);
}

/// Base structure for common list query fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseListQuery {
    pub page: u64,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
}

impl Default for BaseListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            search: None,
            sorts: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        }
    }
}

impl BaseListQuery {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ListQuery for BaseListQuery {
    fn new() -> Self {
        Self::new()
    }

    fn page(&self) -> u64 {
        self.page
    }

    fn set_page(&mut self, page: u64) {
        self.page = page;
    }

    fn search(&self) -> Option<String> {
        self.search.clone()
    }

    fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    fn sorts(&self) -> Option<Vec<SortParam>> {
        self.sorts.clone()
    }

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>) {
        self.sorts = sorts;
    }
}

