//! Integration tests for the ruxlog API.
//!
//! Tests public endpoints and error handling against a live API server.
//! Run with: cargo test --test api_integration
//!
//! Requires the API server running on the port specified by BASE_URL
//! (default: http://127.0.0.1:1100) with a migrated database.
//!
//! CSRF: the per-session token (plan Phase 5) is HMAC-bound to the session id,
//! so these tests bootstrap a real token from `/csrf/v1/generate` and carry it
//! in a cookie-storing reqwest `Client`, keeping the session cookie and its
//! token paired. The previous static shared-secret token is gone.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

const BASE_URL: &str = "http://127.0.0.1:1100";

/// A cookie-storing client so the session cookie issued by
/// `/csrf/v1/generate` is replayed on later requests — the token is only valid
/// alongside the session it was minted for.
fn client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn is_server_up(client: &Client) -> bool {
    client
        .get(format!("{BASE_URL}/healthz"))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

macro_rules! skip_if_no_server {
    ($client:expr) => {
        if !is_server_up(&$client).await {
            eprintln!("SKIP: API server not running on {BASE_URL}");
            return;
        }
    };
}

/// Bootstrap a real per-session CSRF token via the exempt `/csrf/v1/generate`
/// endpoint. The session cookie lands on `client`'s cookie jar; the returned
/// token is bound to that session and must accompany every mutating request.
/// Returns `None` if the server is unreachable or misbehaves.
async fn bootstrap_csrf(client: &Client) -> Option<String> {
    let resp = client
        .post(format!("{BASE_URL}/csrf/v1/generate"))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    body.get("token").and_then(|t| t.as_str()).map(String::from)
}

/// Bootstrap and unwrap, for tests that have already confirmed the server is up.
async fn require_csrf(client: &Client) -> String {
    bootstrap_csrf(client)
        .await
        .expect("server is up but /csrf/v1/generate did not return a token")
}

async fn post_api(client: &Client, path: &str, body: Value, token: &str) -> reqwest::Response {
    client
        .post(format!("{BASE_URL}{path}"))
        .header("csrf-token", token)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .unwrap()
}

async fn get_api(client: &Client, path: &str) -> reqwest::Response {
    client
        .get(format!("{BASE_URL}{path}"))
        .send()
        .await
        .unwrap()
}

// --- Health Check ---

#[tokio::test]
async fn healthz_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = client
        .get(format!("{BASE_URL}/healthz"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// --- Public Category Endpoints ---

#[tokio::test]
async fn category_list_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/category/v1/list").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body.is_array(), "category list should return a JSON array");
}

#[tokio::test]
async fn category_view_with_invalid_id_returns_not_found() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/category/v1/view/999999").await;
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST,
        "expected 404 or 400 for nonexistent category, got {}",
        resp.status()
    );
}

// --- Public Tag Endpoints ---

#[tokio::test]
async fn tag_list_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/tag/v1/list").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body.is_array(), "tag list should return a JSON array");
}

#[tokio::test]
async fn tag_view_with_invalid_id_returns_not_found() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/tag/v1/view/999999").await;
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST,
        "expected 404 or 400 for nonexistent tag, got {}",
        resp.status()
    );
}

// --- Public Post Endpoints ---

#[tokio::test]
async fn post_list_published_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(
        &client,
        "/post/v1/list/published",
        json!({"page": 1}),
        &token,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("data").is_some() || body.is_array(),
        "published posts should have data"
    );
}

#[tokio::test]
async fn post_view_with_invalid_slug_returns_not_found() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(
        &client,
        "/post/v1/view/nonexistent-slug-xyz",
        json!({}),
        &token,
    )
    .await;
    assert!(
        resp.status() == StatusCode::NOT_FOUND,
        "expected 404 for nonexistent post slug, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn post_sitemap_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(&client, "/post/v1/sitemap", json!({}), &token).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- Feed Endpoints ---

#[tokio::test]
async fn rss_feed_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/feed/v1/rss").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("xml") || content_type.contains("rss"),
        "RSS feed should return XML content type, got: {content_type}"
    );
}

#[tokio::test]
async fn atom_feed_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/feed/v1/atom").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("xml") || content_type.contains("atom"),
        "Atom feed should return XML content type, got: {content_type}"
    );
}

// --- Static Routes ---

#[tokio::test]
async fn robots_txt_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/robots.txt").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn sitemap_xml_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/sitemap.xml").await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- CSRF Protection ---

/// A mutating request with no session and no token header is rejected (401).
#[tokio::test]
async fn post_without_csrf_token_returns_unauthorized() {
    let client = client();
    skip_if_no_server!(client);
    let resp = client
        .post(format!("{BASE_URL}/post/v1/list/published"))
        .header("Content-Type", "application/json")
        .json(&json!({"page": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "POST without CSRF token should return 401"
    );
}

/// A mutating request carrying a token that does NOT match its session is
/// rejected (401) — even though a valid session exists. This exercises the
/// token-mismatch path, not merely the missing-session path.
#[tokio::test]
async fn post_with_invalid_csrf_token_returns_unauthorized() {
    let client = client();
    skip_if_no_server!(client);
    // Bootstrap a real session so the rejection is due to a bad token, not a
    // missing session.
    require_csrf(&client).await;
    let resp = client
        .post(format!("{BASE_URL}/post/v1/list/published"))
        .header("csrf-token", "this-token-does-not-match-the-session")
        .header("Content-Type", "application/json")
        .json(&json!({"page": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "POST with wrong CSRF token should return 401"
    );
}

// --- Auth Error Handling ---

#[tokio::test]
async fn login_with_invalid_credentials_returns_unauthorized() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = client
        .post(format!("{BASE_URL}/auth/v1/log_in"))
        .header("csrf-token", token)
        .header("Content-Type", "application/json")
        .json(&json!({"email": "nonexistent@test.com", "password": "wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "login with invalid creds should return 401"
    );
}

#[tokio::test]
async fn login_with_missing_fields_returns_error() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = client
        .post(format!("{BASE_URL}/auth/v1/log_in"))
        .header("csrf-token", token)
        .header("Content-Type", "application/json")
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "login with missing fields should return 400 or 422, got {}",
        resp.status()
    );
}

// --- Protected Endpoints Require Auth ---

#[tokio::test]
async fn post_create_requires_authentication() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(
        &client,
        "/post/v1/create",
        json!({"title": "test", "content": "test", "slug": "test"}),
        &token,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "post create without auth should return 401"
    );
}

#[tokio::test]
async fn category_create_requires_admin_auth() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(
        &client,
        "/category/v1/create",
        json!({"name": "test-cat", "slug": "test-cat"}),
        &token,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "category create without auth should return 401"
    );
}

#[tokio::test]
async fn tag_create_requires_admin_auth() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(
        &client,
        "/tag/v1/create",
        json!({"name": "test-tag", "slug": "test-tag"}),
        &token,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "tag create without auth should return 401"
    );
}

// --- Search ---

#[tokio::test]
async fn search_with_query_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(
        &client,
        "/search/v1/search",
        json!({"query": "rust"}),
        &token,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- API Error Response Format ---

#[tokio::test]
async fn api_errors_have_consistent_json_format() {
    let client = client();
    skip_if_no_server!(client);
    let token = require_csrf(&client).await;
    let resp = post_api(&client, "/post/v1/view/nonexistent", json!({}), &token).await;
    if resp.status() == StatusCode::NOT_FOUND {
        let body: Value = resp.json().await.unwrap();
        assert!(
            body.get("message").is_some()
                || body.get("error").is_some()
                || body.get("type").is_some(),
            "error response should have message/error/type field, got: {body}"
        );
    }
}
