use crate::services::http_client::{HttpError, HttpRequest, HttpResponse};
use oxstore::TransportErrorKind;
use serde::de::DeserializeOwned;
use std::future::Future;

// Adapter wrapper for HttpResponse to implement oxstore trait
pub struct HttpResponseAdapter(pub HttpResponse);

impl oxstore::http::HttpResponse for HttpResponseAdapter {
    fn status(&self) -> u16 {
        gloo_net::http::Response::status(&self.0)
    }

    async fn text(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        gloo_net::http::Response::text(&self.0)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    async fn json<T: DeserializeOwned>(
        &self,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        gloo_net::http::Response::json(&self.0)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

// Transport error classification for HttpError
pub fn classify_transport_error(error: &HttpError) -> (TransportErrorKind, String) {
    use oxstore::classify_transport_error as classify_error;
    let (kind, msg) = classify_error(error);
    (TransportErrorKind::Offline, msg) // This is a placeholder - the real logic is in classify_error
}

// Convenience functions that wrap the oxstore abstractions with the adapters
pub async fn list_state_abstraction<T>(
    state: &oxstore::GlobalSignal<oxstore::StateFrame<T>>,
    req: HttpRequest,
    parse_label: &str,
) -> Option<T>
where
    T: DeserializeOwned + Clone + 'static,
{
    oxstore::abstractions::list_state_abstraction(
        state,
        async move {
            req.send()
                .await
                .map(|r| HttpResponseAdapter(r))
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        parse_label,
    )
    .await
}

pub async fn state_request_abstraction<Data, Meta, Parsed, F, OnSuccess>(
    state: &oxstore::GlobalSignal<oxstore::StateFrame<Data, Meta>>,
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
    oxstore::abstractions::state_request_abstraction(
        state,
        meta,
        async move {
            send_future
                .await
                .map(|r| HttpResponseAdapter(r))
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        parse_label,
        on_success,
    )
    .await
}

pub async fn view_state_abstraction<K, StoreData, Parsed, F, MapFn>(
    state: &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame<StoreData>>>,
    id: K,
    send_future: F,
    parse_label: &str,
    map_to_store: MapFn,
) -> Option<Parsed>
where
    K: std::hash::Hash + Eq + Copy + 'static,
    StoreData: Clone + 'static,
    Parsed: DeserializeOwned + Clone + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    MapFn: Fn(&Parsed) -> StoreData,
{
    oxstore::abstractions::view_state_abstraction(
        state,
        id,
        async move {
            send_future
                .await
                .map(|r| HttpResponseAdapter(r))
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        parse_label,
        map_to_store,
    )
    .await
}

pub async fn edit_state_abstraction<K, T, Payload, F, GetId, OnSuccess>(
    state: &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame<(), Payload>>>,
    id: K,
    payload: Payload,
    send_future: F,
    parse_label: &str,
    sync_list_cache: Option<&oxstore::GlobalSignal<oxstore::StateFrame<oxstore::PaginatedList<T>>>>,
    sync_view_cache: Option<
        &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame<T>>>,
    >,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> Option<T>
where
    K: std::hash::Hash + Eq + Copy + 'static,
    T: DeserializeOwned + Clone + PartialEq + 'static,
    Payload: Clone + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(&T),
{
    oxstore::abstractions::edit_state_abstraction(
        state,
        id,
        payload,
        async move {
            send_future
                .await
                .map(|r| HttpResponseAdapter(r))
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        parse_label,
        sync_list_cache,
        sync_view_cache,
        get_id,
        on_success,
    )
    .await
}

pub async fn remove_state_abstraction<K, T, F, GetId, OnSuccess>(
    state: &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame>>,
    id: K,
    send_future: F,
    _parse_label: &str,
    sync_list_cache: Option<&oxstore::GlobalSignal<oxstore::StateFrame<oxstore::PaginatedList<T>>>>,
    sync_view_cache: Option<
        &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame<T>>>,
    >,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> bool
where
    K: std::hash::Hash + Eq + Copy + 'static,
    T: Clone + PartialEq + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(),
{
    oxstore::abstractions::remove_state_abstraction(
        state,
        id,
        async move {
            send_future
                .await
                .map(|r| HttpResponseAdapter(r))
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        _parse_label,
        sync_list_cache,
        sync_view_cache,
        get_id,
        on_success,
    )
    .await
}

pub async fn remove_state_abstraction_vec<K, T, F, GetId, OnSuccess>(
    state: &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame>>,
    id: K,
    send_future: F,
    _parse_label: &str,
    sync_list_cache: Option<&oxstore::GlobalSignal<oxstore::StateFrame<Vec<T>>>>,
    sync_view_cache: Option<
        &oxstore::GlobalSignal<std::collections::HashMap<K, oxstore::StateFrame<Option<T>>>>,
    >,
    get_id: GetId,
    on_success: Option<OnSuccess>,
) -> bool
where
    K: std::hash::Hash + Eq + Copy + 'static,
    T: Clone + PartialEq + 'static,
    F: Future<Output = Result<HttpResponse, HttpError>>,
    GetId: Fn(&T) -> K,
    OnSuccess: FnOnce(),
{
    oxstore::abstractions::remove_state_abstraction_vec(
        state,
        id,
        async move {
            send_future
                .await
                .map(|r| HttpResponseAdapter(r))
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        _parse_label,
        sync_list_cache,
        sync_view_cache,
        get_id,
        on_success,
    )
    .await
}
