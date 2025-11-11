use crate::services::http_client::{HttpError, HttpRequest, HttpResponse};
use crate::store::error::{
    classify_transport_error, ApiError, AppError, TransportErrorInfo, TransportErrorKind,
};
use dioxus::logger::tracing;
use dioxus::prelude::GlobalSignal;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json;
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;

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

    pub async fn set_api_error(&mut self, response: &HttpResponse) {
        self.status = StateFrameStatus::Failed;
        let status = response.status();
        let body = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                self.error = Some(AppError::Decode {
                    label: "api_error".to_string(),
                    error: format!("Failed to read API error body: {}", e),
                    raw: None,
                });
                return;
            }
        };

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

    /// Mark this frame as a transport-layer failure (network/offline/etc.)
    pub fn set_transport_error(&mut self, kind: TransportErrorKind, message: Option<String>) {
        self.status = StateFrameStatus::Failed;
        self.error = Some(AppError::Transport(TransportErrorInfo { kind, message }));
    }

    /// Mark this frame as a decode/serialization error for a successful HTTP response
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

/// Send a request, parse JSON into `T`, and update the provided `StateFrame<T>`.
/// Returns `Some(T)` on success to allow callers to perform cache-sync logic if needed.
pub async fn list_state_abstraction<T>(
    state: &GlobalSignal<StateFrame<T>>,
    req: HttpRequest,
    parse_label: &str,
) -> Option<T>
where
    T: DeserializeOwned + Clone + 'static,
{
    state.write().set_loading();
    match req.send().await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                match response.json::<T>().await {
                    Ok(data) => {
                        state.write().set_success(Some(data.clone()));
                        Some(data)
                    }
                    Err(e) => {
                        // Try to get the raw response text for better debugging
                        let response_text = response.text().await.unwrap_or_default();
                        tracing::error!(
                            "Failed to parse {}: {:?}\nResponse: {}",
                            parse_label,
                            e,
                            response_text
                        );
                        state.write().set_decode_error(
                            parse_label,
                            format!("{}", e),
                            Some(response_text),
                        );
                        None
                    }
                }
            } else {
                state.write().set_api_error(&response).await;
                None
            }
        }
        Err(e) => {
            let (kind, msg) = classify_transport_error(&e);
            state.write().set_transport_error(kind, Some(msg));
            None
        }
    }
}

/// Generic helper for request/response cycles that update a single `StateFrame`.
/// Returns `Some(Parsed)` on success so callers can chain follow-up actions.
pub async fn state_request_abstraction<Data, Meta, Parsed, F, OnSuccess>(
    state: &GlobalSignal<StateFrame<Data, Meta>>,
    meta: Option<Meta>,
    send_future: F,
    parse_label: &str,
    on_success: OnSuccess,
) -> Option<Parsed>
where
    Data: Clone + 'static,
    Meta: Clone + 'static,
    Parsed: DeserializeOwned + Clone + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    OnSuccess: Fn(&Parsed) -> (Option<Data>, Option<String>),
{
    {
        let mut frame = state.write();
        frame.set_loading_meta(meta);
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                match response.json::<Parsed>().await {
                    Ok(parsed) => {
                        let (data, _message) = on_success(&parsed);
                        state.write().set_success(data);
                        Some(parsed)
                    }
                    Err(e) => {
                        let response_text = response.text().await.unwrap_or_default();
                        state.write().set_decode_error(
                            parse_label,
                            format!("{}", e),
                            Some(response_text),
                        );
                        None
                    }
                }
            } else {
                state.write().set_api_error(&response).await;
                None
            }
        }
        Err(e) => {
            let (kind, msg) = classify_transport_error(&e);
            state.write().set_transport_error(kind, Some(msg));
            None
        }
    }
}

/// Shared helper to fetch a single record and hydrate a keyed `StateFrame` map.
/// Returns `Some(Parsed)` on success so callers can optionally sync additional caches.
pub async fn view_state_abstraction<K, StoreData, Parsed, F, MapFn>(
    state: &GlobalSignal<HashMap<K, StateFrame<StoreData>>>,
    id: K,
    send_future: F,
    parse_label: &str,
    map_to_store: MapFn,
) -> Option<Parsed>
where
    K: Eq + Hash + Copy + 'static,
    StoreData: Clone + 'static,
    Parsed: DeserializeOwned + Clone + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    MapFn: Fn(&Parsed) -> StoreData,
{
    {
        let mut map = state.write();
        map.entry(id).or_insert_with(StateFrame::new).set_loading();
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                match response.json::<Parsed>().await {
                    Ok(parsed) => {
                        let store_value = map_to_store(&parsed);
                        let mut map = state.write();
                        map.entry(id)
                            .or_insert_with(StateFrame::new)
                            .set_success(Some(store_value));
                        Some(parsed)
                    }
                    Err(e) => {
                        let response_text = response.text().await.unwrap_or_default();
                        tracing::error!(
                            "Failed to parse {}: {}\nResponse: {}",
                            parse_label,
                            e,
                            response_text
                        );
                        let mut map = state.write();
                        map.entry(id)
                            .or_insert_with(StateFrame::new)
                            .set_decode_error(parse_label, format!("{}", e), Some(response_text));
                        None
                    }
                }
            } else {
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(&response)
                    .await;
                None
            }
        }
        Err(e) => {
            let (kind, msg) = classify_transport_error(&e);
            let mut map = state.write();
            map.entry(id)
                .or_insert_with(StateFrame::new)
                .set_transport_error(kind, Some(msg));
            None
        }
    }
}

/// Specialized version for updating items in a PaginatedList cache
pub async fn edit_state_abstraction<K, T, Payload, F, GetId, OnSuccess>(
    state: &GlobalSignal<HashMap<K, StateFrame<(), Payload>>>,
    id: K,
    payload: Payload,
    send_future: F,
    parse_label: &str,
    sync_list_cache: Option<&GlobalSignal<StateFrame<PaginatedList<T>>>>,
    sync_view_cache: Option<&GlobalSignal<HashMap<K, StateFrame<T>>>>,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> Option<T>
where
    K: Eq + Hash + Copy + 'static,
    T: DeserializeOwned + Clone + PartialEq + 'static,
    Payload: Clone + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(&T),
{
    {
        let mut map = state.write();
        map.entry(id)
            .or_insert_with(StateFrame::new)
            .set_loading_meta(Some(payload));
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                match response.json::<T>().await {
                    Ok(parsed) => {
                        {
                            let mut map = state.write();
                            map.entry(id)
                                .or_insert_with(StateFrame::new)
                                .set_success(None);
                        }

                        // Sync list cache if provided
                        if let Some(list_cache) = sync_list_cache {
                            let mut list_frame = list_cache.write();
                            if let Some(list) = &mut list_frame.data {
                                if let Some(item) = list.data.iter_mut().find(|i| get_id(i) == id) {
                                    *item = parsed.clone();
                                }
                            }
                        }

                        // Sync view cache if provided
                        if let Some(view_cache) = sync_view_cache {
                            let mut view_map = view_cache.write();
                            view_map
                                .entry(id)
                                .or_insert_with(StateFrame::new)
                                .set_success(Some(parsed.clone()));
                        }

                        // Call optional success callback for custom logic
                        if let Some(callback) = on_success {
                            callback(&parsed);
                        }

                        Some(parsed)
                    }
                    Err(e) => {
                        let response_text = response.text().await.unwrap_or_default();
                        let mut map = state.write();
                        map.entry(id)
                            .or_insert_with(StateFrame::new)
                            .set_decode_error(parse_label, format!("{}", e), Some(response_text));
                        None
                    }
                }
            } else {
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(&response)
                    .await;
                None
            }
        }
        Err(e) => {
            let (kind, msg) = classify_transport_error(&e);
            let mut map = state.write();
            map.entry(id)
                .or_insert_with(StateFrame::new)
                .set_transport_error(kind, Some(msg));
            None
        }
    }
}

/// Specialized version for removing items and syncing caches
/// Use this when sync_view_cache stores `StateFrame<T>` (not wrapped in Option)
pub async fn remove_state_abstraction<K, T, F, GetId, OnSuccess>(
    state: &GlobalSignal<HashMap<K, StateFrame>>,
    id: K,
    send_future: F,
    _parse_label: &str,
    sync_list_cache: Option<&GlobalSignal<StateFrame<PaginatedList<T>>>>,
    sync_view_cache: Option<&GlobalSignal<HashMap<K, StateFrame<T>>>>,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> bool
where
    K: Eq + Hash + Copy + 'static,
    T: Clone + PartialEq + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(),
{
    {
        let mut map = state.write();
        map.entry(id).or_insert_with(StateFrame::new).set_loading();
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                {
                    let mut map = state.write();
                    map.entry(id)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);
                }

                // Sync list cache if provided - remove the item
                if let Some(list_cache) = sync_list_cache {
                    let mut list_frame = list_cache.write();
                    if let Some(list) = &mut list_frame.data {
                        list.data.retain(|item| get_id(item) != id);
                        // Update total count
                        if list.total > 0 {
                            list.total -= 1;
                        }
                    }
                }

                // Sync view cache if provided - remove the entry
                if let Some(view_cache) = sync_view_cache {
                    let mut view_map = view_cache.write();
                    view_map.remove(&id);
                }

                // Call optional success callback for custom logic
                if let Some(callback) = on_success {
                    callback();
                }

                true
            } else {
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(&response)
                    .await;
                false
            }
        }
        Err(e) => {
            let (kind, msg) = classify_transport_error(&e);
            let mut map = state.write();
            map.entry(id)
                .or_insert_with(StateFrame::new)
                .set_transport_error(kind, Some(msg));
            false
        }
    }
}

/// Variant for Vec-based lists instead of PaginatedList
/// Use this when sync_view_cache stores `StateFrame<Option<T>>`
pub async fn remove_state_abstraction_vec<K, T, F, GetId, OnSuccess>(
    state: &GlobalSignal<HashMap<K, StateFrame>>,
    id: K,
    send_future: F,
    _parse_label: &str,
    sync_list_cache: Option<&GlobalSignal<StateFrame<Vec<T>>>>,
    sync_view_cache: Option<&GlobalSignal<HashMap<K, StateFrame<Option<T>>>>>,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> bool
where
    K: Eq + Hash + Copy + 'static,
    T: Clone + PartialEq + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(),
{
    {
        let mut map = state.write();
        map.entry(id).or_insert_with(StateFrame::new).set_loading();
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                {
                    let mut map = state.write();
                    map.entry(id)
                        .or_insert_with(StateFrame::new)
                        .set_success(None);
                }

                // Sync list cache if provided - remove the item
                if let Some(list_cache) = sync_list_cache {
                    let mut list_frame = list_cache.write();
                    if let Some(list) = &mut list_frame.data {
                        list.retain(|item| get_id(item) != id);
                    }
                }

                // Sync view cache if provided - remove the entry
                if let Some(view_cache) = sync_view_cache {
                    let mut view_map = view_cache.write();
                    view_map.remove(&id);
                }

                // Call optional success callback for custom logic
                if let Some(callback) = on_success {
                    callback();
                }

                true
            } else {
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(&response)
                    .await;
                false
            }
        }
        Err(e) => {
            let (kind, msg) = classify_transport_error(&e);
            let mut map = state.write();
            map.entry(id)
                .or_insert_with(StateFrame::new)
                .set_transport_error(kind, Some(msg));
            false
        }
    }
}
