//! Integration tests for the ruxlog API.
//!
//! Tests public endpoints and error handling against a live API server.
//! Run with: cargo test --test api_integration
//!
//! Requires the API server running on the port specified by API_BASE_URL
//! (default: http://127.0.0.1:1100) with a migrated database.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

const BASE_URL: &str = "http://127.0.0.1:1100";
const CSRF_TOKEN: &str = "dWx0cmEtaW5zdGluY3QtZ29rdQ=="; // base64("ultra-instinct-goku")

fn client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
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

async fn post_api(client: &Client, path: &str, body: Value) -> reqwest::Response {
    client
        .post(format!("{BASE_URL}{path}"))
        .header("csrf-token", CSRF_TOKEN)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .unwrap()
}

async fn get_api(client: &Client, path: &str) -> reqwest::Response {
    client
        .get(format!("{BASE_URL}{path}"))
        .header("csrf-token", CSRF_TOKEN)
        .send()
        .await
        .unwrap()
}

// --- Health Check ---

#[tokio::test]
async fn healthz_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = client.get(format!("{BASE_URL}/healthz")).send().await.unwrap();
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
    let resp = post_api(&client, "/post/v1/list/published", json!({"page": 1})).await;
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
    let resp = post_api(&client, "/post/v1/view/nonexistent-slug-xyz", json!({})).await;
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
    let resp = post_api(&client, "/post/v1/sitemap", json!({})).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- Feed Endpoints ---

#[tokio::test]
async fn rss_feed_returns_ok() {
    let client = client();
    skip_if_no_server!(client);
    let resp = get_api(&client, "/feed/v1/rss").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
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
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
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

#[tokio::test]
async fn post_with_invalid_csrf_token_returns_unauthorized() {
    let client = client();
    skip_if_no_server!(client);
    let resp = client
        .post(format!("{BASE_URL}/post/v1/list/published"))
        .header("csrf-token", "invalid-token")
        .header("Content-Type", "application/json")
        .json(&json!({"page": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "POST with invalid CSRF token should return 401"
    );
}

// --- Auth Error Handling ---

#[tokio::test]
async fn login_with_invalid_credentials_returns_unauthorized() {
    let client = client();
    skip_if_no_server!(client);
    let resp = client
        .post(format!("{BASE_URL}/auth/v1/log_in"))
        .header("csrf-token", CSRF_TOKEN)
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
    let resp = client
        .post(format!("{BASE_URL}/auth/v1/log_in"))
        .header("csrf-token", CSRF_TOKEN)
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
    let resp = post_api(
        &client,
        "/post/v1/create",
        json!({"title": "test", "content": "test", "slug": "test"}),
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
    let resp = post_api(
        &client,
        "/category/v1/create",
        json!({"name": "test-cat", "slug": "test-cat"}),
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
    let resp = post_api(
        &client,
        "/tag/v1/create",
        json!({"name": "test-tag", "slug": "test-tag"}),
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
    let resp = post_api(&client, "/search/v1/search", json!({"query": "rust"})).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- API Error Response Format ---

#[tokio::test]
async fn api_errors_have_consistent_json_format() {
    let client = client();
    skip_if_no_server!(client);
    let resp = post_api(&client, "/post/v1/view/nonexistent", json!({})).await;
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
