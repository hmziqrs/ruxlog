use crate::error::classify_transport_error;
use crate::pagination::PaginatedList;
use crate::state::StateFrame;
use dioxus::prelude::GlobalSignal;
use oxcore::http;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;

/// Send a request, parse JSON into `T`, and update the provided `StateFrame<T>`.
/// Returns `Some(T)` on success to allow callers to perform cache-sync logic if needed.
pub async fn list_state_abstraction<T, Req>(
    state: &GlobalSignal<StateFrame<T>>,
    request: Req,
    parse_label: &str,
) -> Option<T>
where
    T: DeserializeOwned + Clone + 'static,
    Req: Future<Output = Result<http::Response, http::Error>>,
{
    state.write().set_loading();
    match request.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                let body_text = response.body_text();
                match response.json::<T>().await {
                    Ok(data) => {
                        state.write().set_success(Some(data.clone()));
                        Some(data)
                    }
                    Err(e) => {
                        dioxus::logger::tracing::error!(
                            "Failed to parse {}: {:?}\nResponse: {}",
                            parse_label,
                            e,
                            body_text
                        );
                        state.write().set_decode_error(
                            parse_label,
                            format!("{}", e),
                            Some(body_text),
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
    Data: DeserializeOwned + Clone + 'static,
    Meta: Clone + 'static,
    Parsed: DeserializeOwned + Clone + 'static,
    F: Future<Output = Result<http::Response, http::Error>>,
    OnSuccess: Fn(&Parsed) -> (Option<Data>, Option<String>),
{
    {
        let mut frame = state.write();
        frame.set_loading_meta(meta);
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                let body_text = response.body_text();
                match response.json::<Parsed>().await {
                    Ok(parsed) => {
                        let (data, _message) = on_success(&parsed);
                        state.write().set_success(data);
                        Some(parsed)
                    }
                    Err(e) => {
                        state.write().set_decode_error(
                            parse_label,
                            format!("{}", e),
                            Some(body_text),
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
    F: Future<Output = Result<http::Response, http::Error>>,
    MapFn: Fn(&Parsed) -> StoreData,
{
    {
        let mut map = state.write();
        map.entry(id).or_insert_with(StateFrame::new).set_loading();
    }

    match send_future.await {
        Ok(response) => {
            if (200..300).contains(&response.status()) {
                let body_text = response.body_text();
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
                        dioxus::logger::tracing::error!(
                            "Failed to parse {}: {}\nResponse: {}",
                            parse_label,
                            e,
                            body_text
                        );
                        let mut map = state.write();
                        map.entry(id)
                            .or_insert_with(StateFrame::new)
                            .set_decode_error(parse_label, format!("{}", e), Some(body_text));
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
    F: Future<Output = Result<http::Response, http::Error>>,
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
                let body_text = response.body_text();
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
                        let mut map = state.write();
                        map.entry(id)
                            .or_insert_with(StateFrame::new)
                            .set_decode_error(parse_label, format!("{}", e), Some(body_text));
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
    F: Future<Output = Result<http::Response, http::Error>>,
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
    F: Future<Output = Result<http::Response, http::Error>>,
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
