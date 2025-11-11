use crate::services::http_client::{HttpError, HttpRequest, HttpResponse};
use dioxus::logger::tracing;
use dioxus::prelude::GlobalSignal;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;

// Re-export all the types from oxstore
pub use oxstore::*;

// Use classify_transport_error from the error module
pub use crate::store::error::classify_transport_error as classify_transport_error_impl;

// Convert TransportErrorKind from error module to oxstore version
fn convert_transport_error_kind(kind: crate::store::error::TransportErrorKind) -> TransportErrorKind {
    match kind {
        crate::store::error::TransportErrorKind::Offline => TransportErrorKind::Offline,
        crate::store::error::TransportErrorKind::Network => TransportErrorKind::Network,
        crate::store::error::TransportErrorKind::Timeout => TransportErrorKind::Timeout,
        crate::store::error::TransportErrorKind::Canceled => TransportErrorKind::Canceled,
        crate::store::error::TransportErrorKind::Unknown => TransportErrorKind::Unknown,
    }
}

pub fn classify_transport_error(e: &HttpError) -> (TransportErrorKind, String) {
    let (kind, msg) = classify_transport_error_impl(e);
    (convert_transport_error_kind(kind), msg)
}

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
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                state.write().set_api_error(status, body);
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
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                state.write().set_api_error(status, body);
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
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(status, body);
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

                        if let Some(list_cache) = sync_list_cache {
                            let mut list_frame = list_cache.write();
                            if let Some(list) = &mut list_frame.data {
                                if let Some(item) = list.data.iter_mut().find(|i| get_id(i) == id) {
                                    *item = parsed.clone();
                                }
                            }
                        }

                        if let Some(view_cache) = sync_view_cache {
                            let mut view_map = view_cache.write();
                            view_map
                                .entry(id)
                                .or_insert_with(StateFrame::new)
                                .set_success(Some(parsed.clone()));
                        }

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
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(status, body);
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

                if let Some(list_cache) = sync_list_cache {
                    let mut list_frame = list_cache.write();
                    if let Some(list) = &mut list_frame.data {
                        list.data.retain(|item| get_id(item) != id);
                        if list.total > 0 {
                            list.total -= 1;
                        }
                    }
                }

                if let Some(view_cache) = sync_view_cache {
                    let mut view_map = view_cache.write();
                    view_map.remove(&id);
                }

                if let Some(callback) = on_success {
                    callback();
                }

                true
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(status, body);
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

                if let Some(list_cache) = sync_list_cache {
                    let mut list_frame = list_cache.write();
                    if let Some(list) = &mut list_frame.data {
                        list.retain(|item| get_id(item) != id);
                    }
                }

                if let Some(view_cache) = sync_view_cache {
                    let mut view_map = view_cache.write();
                    view_map.remove(&id);
                }

                if let Some(callback) = on_success {
                    callback();
                }

                true
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let mut map = state.write();
                map.entry(id)
                    .or_insert_with(StateFrame::new)
                    .set_api_error(status, body);
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