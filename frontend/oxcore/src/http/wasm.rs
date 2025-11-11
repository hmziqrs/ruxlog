use super::config::{get_base_url, get_csrf_token};
use gloo_net::http::{Request as GlooRequest, RequestBuilder as GlooRequestBuilder};
use serde::de::Error as _;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::pin::Pin;
use web_sys::{FormData, RequestCredentials};

pub struct Request(GlooRequest);
pub struct RequestBuilder(GlooRequestBuilder);

impl Request {
    pub async fn send(self) -> Result<Response, Error> {
        let resp = self.0.send().await?;
        Response::from_gloo(resp).await
    }
}

impl RequestBuilder {
    pub async fn send(self) -> Result<Response, Error> {
        let resp = self.0.send().await?;
        Response::from_gloo(resp).await
    }
}

/// Unified error type for HTTP operations
#[derive(Debug)]
pub struct Error(pub gloo_net::Error);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<gloo_net::Error> for Error {
    fn from(e: gloo_net::Error) -> Self {
        Error(e)
    }
}

/// Unified response wrapper that provides a clean API for both platforms
pub struct Response {
    status: u16,
    body: Vec<u8>,
}

impl Response {
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get response body as text without consuming self
    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    pub async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.body).map_err(|e| {
            Error(gloo_net::Error::SerdeError(serde_json::Error::custom(format!(
                "UTF-8 decode error: {}",
                e
            ))))
        })
    }

    pub async fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
        serde_json::from_slice(&self.body).map_err(|e| Error(gloo_net::Error::SerdeError(e)))
    }
}

impl Response {
    pub async fn from_gloo(resp: gloo_net::http::Response) -> Result<Self, Error> {
        let status = resp.status();
        let body = resp.binary().await.map_err(Error)?;
        Ok(Response { status, body })
    }
}

// ============================================================================
// HTTP Helper Functions
// ============================================================================

fn create_headers(mut req: GlooRequestBuilder) -> GlooRequestBuilder {
    req = req
        .header("Content-Type", "application/json")
        .header("csrf-token", &get_csrf_token())
        .credentials(RequestCredentials::Include);
    req
}

pub fn get(endpoint: &str) -> RequestBuilder {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req = GlooRequest::get(&url);
    RequestBuilder(create_headers(req))
}

pub fn post<T: Serialize>(endpoint: &str, body: &T) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req_pre = GlooRequest::post(&url);
    let req = create_headers(req_pre).json(body).unwrap();
    Request(req)
}

pub fn put<T: Serialize>(endpoint: &str, body: &T) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req_pre = GlooRequest::put(&url);
    let req = create_headers(req_pre).json(body).unwrap();
    Request(req)
}

pub fn delete(endpoint: &str) -> RequestBuilder {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req = GlooRequest::delete(&url);
    RequestBuilder(create_headers(req))
}

fn create_multipart_headers(mut req: GlooRequestBuilder) -> GlooRequestBuilder {
    req = req
        .header("csrf-token", &get_csrf_token())
        .credentials(RequestCredentials::Include);
    // Note: Don't set Content-Type for multipart, browser will set it with boundary
    req
}

pub fn post_multipart(endpoint: &str, form_data: &FormData) -> Result<Request, String> {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req_pre = GlooRequest::post(&url);
    let req_builder = create_multipart_headers(req_pre);

    let req = req_builder
        .body(form_data)
        .map_err(|e| format!("Failed to create multipart request: {:?}", e))?;

    Ok(Request(req))
}
