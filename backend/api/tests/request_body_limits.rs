use axum::{
    body::{Body, Bytes},
    extract::Multipart,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
mod size_config {
    include!("../src/config.rs");
}

use size_config::body_limits;
use tower::ServiceExt;
use tower_http::limit::RequestBodyLimitLayer;

async fn accept_bytes(_: Bytes) -> StatusCode {
    StatusCode::OK
}

async fn accept_multipart(mut multipart: Multipart) -> StatusCode {
    loop {
        let next_field = match multipart.next_field().await {
            Ok(field) => field,
            Err(_) => return StatusCode::PAYLOAD_TOO_LARGE,
        };

        let Some(field) = next_field else {
            break StatusCode::OK;
        };

        if field.bytes().await.is_err() {
            return StatusCode::PAYLOAD_TOO_LARGE;
        }
    }
}

fn build_router() -> Router {
    Router::new()
        .route("/default", post(accept_bytes))
        .nest(
            "/post/v1",
            Router::new()
                .route("/payload", post(accept_bytes))
                .layer(RequestBodyLimitLayer::new(body_limits::POST)),
        )
        .nest(
            "/media/v1",
            Router::new()
                .route("/upload", post(accept_multipart))
                .layer(RequestBodyLimitLayer::new(body_limits::MEDIA)),
        )
        .layer(RequestBodyLimitLayer::new(body_limits::DEFAULT))
}

fn multipart_body(boundary: &str, filename: &str, bytes: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            filename
        )
        .as_bytes(),
    );
    body.extend(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(bytes);
    body.extend(b"\r\n");
    body.extend(format!("--{}--\r\n", boundary).as_bytes());
    body
}

#[tokio::test]
async fn default_payload_under_limit_is_accepted() {
    let app = build_router();
    let body = vec![b'a'; 60 * 1024];

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/default")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn default_payload_over_limit_is_rejected() {
    let app = build_router();
    let body = vec![b'a'; 70 * 1024];

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/default")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn post_payload_under_limit_is_accepted() {
    let app = build_router();
    let body = std::fs::read("tmp/240kb.json").expect("240kb.json fixture must exist");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/post/v1/payload")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_payload_over_limit_is_rejected() {
    let app = build_router();
    let body = std::fs::read("tmp/260kb.json").expect("260kb.json fixture must exist");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/post/v1/payload")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn media_payload_under_limit_is_accepted() {
    let app = build_router();
    let bytes = std::fs::read("tmp/2mbunder.jpg").expect("2mbunder.jpg fixture must exist");
    let boundary = "BOUNDARY";
    let body = multipart_body(boundary, "2mbunder.jpg", &bytes);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/media/v1/upload")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn media_payload_over_limit_is_rejected() {
    let app = build_router();
    let bytes = std::fs::read("tmp/2mbplus.jpg").expect("2mbplus.jpg fixture must exist");
    let boundary = "BOUNDARY";
    let body = multipart_body(boundary, "2mbplus.jpg", &bytes);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/media/v1/upload")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
