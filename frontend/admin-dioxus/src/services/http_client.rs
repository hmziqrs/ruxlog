use crate::env::{APP_API_URL, APP_CSRF_TOKEN};
use gloo_net::http::{Request, RequestBuilder, Response};
use serde::Serialize;
use web_sys::{FormData, RequestCredentials};

pub type HttpRequest = Request;
pub type HttpRequestBuilder = RequestBuilder;
pub type HttpResponse = Response;
pub type HttpError = gloo_net::Error;

pub fn get_base_url() -> String {
    format!("http://{}", APP_API_URL)
}

fn create_headers(mut req: RequestBuilder) -> RequestBuilder {
    req = req
        .header("Content-Type", "application/json")
        .header("csrf-token", APP_CSRF_TOKEN)
        .credentials(RequestCredentials::Include);
    req
}

pub fn get(endpoint: &str) -> RequestBuilder {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req = Request::get(&url);
    create_headers(req)
}

pub fn post<T: Serialize>(endpoint: &str, body: &T) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req_pre = Request::post(&url);
    let req = create_headers(req_pre).json(body).unwrap();
    req
}

pub fn put<T: Serialize>(endpoint: &str, body: &T) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req_pre = Request::put(&url);
    let req = create_headers(req_pre).json(body).unwrap();
    req
}

pub fn delete(endpoint: &str) -> RequestBuilder {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req = Request::delete(&url);
    create_headers(req)
}

fn create_multipart_headers(mut req: RequestBuilder) -> RequestBuilder {
    req = req
        .header("csrf-token", APP_CSRF_TOKEN)
        .credentials(RequestCredentials::Include);
    // Note: Don't set Content-Type for multipart, browser will set it with boundary
    req
}

pub fn post_multipart(endpoint: &str, form_data: &FormData) -> Result<Request, String> {
    let url = format!("{}{}", get_base_url(), endpoint);
    let req_pre = Request::post(&url);
    let req_builder = create_multipart_headers(req_pre);

    let req = req_builder
        .body(form_data)
        .map_err(|e| format!("Failed to create multipart request: {:?}", e))?;

    Ok(req)
}
