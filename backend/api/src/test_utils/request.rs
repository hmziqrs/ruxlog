//! HTTP request helpers for tests.
//!
//! In the per-session CSRF scheme, a mutating request must carry a token bound
//! to the *same session* the request belongs to. Callers therefore pass the
//! session id (obtained from `/csrf/v1/generate` in an integration test) so the
//! correct token is attached. For GET/HEAD/OPTIONS the token is irrelevant
//! (safe methods are exempt) but may still be attached harmlessly.

use axum::{
    body::Body,
    http::{Method, Request},
};
use serde_json::Value;

use super::csrf::csrf_header_for_session;

/// Build a JSON POST request to the given path with a CSRF token bound to
/// `session_id`.
pub fn json_post(path: &str, body: Value, session_id: &str) -> Request<Body> {
    let (name, value) = csrf_header_for_session(session_id);
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", "application/json")
        .header(name, value)
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Build a JSON GET request to the given path. CSRF is not enforced on safe
/// methods, so no session id is required.
pub fn json_get(path: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .unwrap()
}

/// Build a raw POST request with arbitrary body bytes and a CSRF token bound to
/// `session_id`.
pub fn raw_post(path: &str, content_type: &str, body: Vec<u8>, session_id: &str) -> Request<Body> {
    let (name, value) = csrf_header_for_session(session_id);
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", content_type)
        .header(name, value)
        .body(Body::from(body))
        .unwrap()
}
