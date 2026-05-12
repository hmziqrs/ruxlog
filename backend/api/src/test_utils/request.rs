//! HTTP request helpers for tests.

use axum::{
    body::Body,
    http::{Method, Request},
};
use serde_json::Value;

/// Build a JSON POST request to the given path with CSRF header.
pub fn json_post(path: &str, body: Value) -> Request<Body> {
    let csrf = super::csrf::test_csrf_token();
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", "application/json")
        .header("csrf-token", csrf)
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Build a JSON GET request to the given path with CSRF header.
pub fn json_get(path: &str) -> Request<Body> {
    let csrf = super::csrf::test_csrf_token();
    Request::builder()
        .method(Method::GET)
        .uri(path)
        .header("csrf-token", csrf)
        .body(Body::empty())
        .unwrap()
}

/// Build a raw POST request with arbitrary body bytes and CSRF header.
pub fn raw_post(path: &str, content_type: &str, body: Vec<u8>) -> Request<Body> {
    let csrf = super::csrf::test_csrf_token();
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", content_type)
        .header("csrf-token", csrf)
        .body(Body::from(body))
        .unwrap()
}
