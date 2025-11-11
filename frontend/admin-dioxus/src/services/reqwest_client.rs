#![cfg(not(target_arch = "wasm32"))]
use crate::env::{APP_API_URL, APP_CSRF_TOKEN};
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE},
    Client, Method, Response,
};
use serde::Serialize;

/// Create the base URL, mirroring `src/services/http_client.rs`.
pub fn get_base_url() -> String {
    format!("http://{}", APP_API_URL)
}

/// Placeholder for auth token retrieval, if needed later.
pub fn get_auth_token() -> Option<String> {
    None
}

/// Global reqwest client with default headers and cookie store enabled.
static CLIENT: Lazy<Client> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        HeaderName::from_static("csrf-token"),
        HeaderValue::from_static(APP_CSRF_TOKEN),
    );

    Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()
        .expect("failed to build reqwest Client")
});

/// Access the singleton client.
pub fn client() -> &'static Client {
    &CLIENT
}

/// Start a request builder with the preconfigured client.
pub fn req(endpoint: &str, method: Method) -> reqwest::RequestBuilder {
    let url = format!("{}{}", get_base_url(), endpoint);
    client().request(method, url)
}

/// Optionally add bearer auth to a request.
pub fn with_bearer(builder: reqwest::RequestBuilder, token: &str) -> reqwest::RequestBuilder {
    builder.bearer_auth(token)
}

pub async fn get(endpoint: &str) -> reqwest::Result<Response> {
    req(endpoint, Method::GET).send().await
}

pub async fn delete(endpoint: &str) -> reqwest::Result<Response> {
    req(endpoint, Method::DELETE).send().await
}

pub async fn post_json<T: Serialize + ?Sized>(
    endpoint: &str,
    body: &T,
) -> reqwest::Result<Response> {
    req(endpoint, Method::POST).json(body).send().await
}

pub async fn put_json<T: Serialize + ?Sized>(
    endpoint: &str,
    body: &T,
) -> reqwest::Result<Response> {
    req(endpoint, Method::PUT).json(body).send().await
}
