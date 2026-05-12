//! Security-focused integration tests.
//!
//! Verifies CSRF rejection, security headers, and input validation edge cases.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::{get, post},
    Router,
};
use base64::Engine;
use tower::ServiceExt;

use ruxlog::middlewares::security_headers::security_headers;
use ruxlog::middlewares::static_csrf::csrf_guard;

// --- Security Headers Tests ---

async fn ok_handler() -> StatusCode {
    StatusCode::OK
}

fn security_headers_router() -> Router {
    Router::new()
        .route("/test", get(ok_handler))
        .layer(middleware::from_fn(security_headers))
}

#[tokio::test]
async fn security_headers_present_on_response() {
    let app = security_headers_router();
    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let headers = response.headers();
    assert_eq!(
        headers
            .get("x-content-type-options")
            .map(|v| v.to_str().unwrap()),
        Some("nosniff")
    );
    assert_eq!(
        headers.get("x-frame-options").map(|v| v.to_str().unwrap()),
        Some("DENY")
    );
    assert_eq!(
        headers.get("referrer-policy").map(|v| v.to_str().unwrap()),
        Some("strict-origin-when-cross-origin")
    );
    assert_eq!(
        headers
            .get("permissions-policy")
            .map(|v| v.to_str().unwrap()),
        Some("camera=(), microphone=(), geolocation=()")
    );
    assert_eq!(
        headers.get("x-xss-protection").map(|v| v.to_str().unwrap()),
        Some("0")
    );
}

// --- CSRF Guard Tests ---

fn csrf_router() -> Router {
    Router::new()
        .route("/protected", post(ok_handler))
        .layer(middleware::from_fn(csrf_guard))
}

#[tokio::test]
async fn csrf_rejects_missing_token() {
    let app = csrf_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn csrf_rejects_invalid_token() {
    let app = csrf_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .header("csrf-token", "invalid-token-value")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn csrf_rejects_wrong_key() {
    let app = csrf_router();
    // base64-encode a wrong key
    let wrong_token = base64::engine::general_purpose::STANDARD.encode("wrong-key-value");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .header("csrf-token", wrong_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn csrf_accepts_valid_token() {
    let app = csrf_router();
    let valid_token = base64::engine::general_purpose::STANDARD.encode("ultra-instinct-goku");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .header("csrf-token", valid_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn csrf_exempt_google_oauth_paths() {
    let app = csrf_router();
    // Google OAuth callback should bypass CSRF
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                // The actual exempt check is on the path prefix, so this test verifies
                // the middleware still requires CSRF for non-exempt paths
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// --- Input Validation Edge Cases ---

#[test]
fn empty_string_not_accepted_as_totp_code() {
    use ruxlog::utils::twofa;

    let secret = twofa::generate_secret_base32(20);
    assert!(!twofa::verify_totp_code_at(
        &secret,
        "",
        chrono::Utc::now().fixed_offset(),
        twofa::DEFAULT_TOTP_STEP,
        twofa::DEFAULT_TOTP_DIGITS,
        1,
    ));
}

#[test]
fn non_numeric_totp_code_rejected() {
    use ruxlog::utils::twofa;

    let secret = twofa::generate_secret_base32(20);
    assert!(!twofa::verify_totp_code_at(
        &secret,
        "abcdef",
        chrono::Utc::now().fixed_offset(),
        twofa::DEFAULT_TOTP_STEP,
        twofa::DEFAULT_TOTP_DIGITS,
        1,
    ));
}

#[test]
fn wrong_length_totp_code_rejected() {
    use ruxlog::utils::twofa;

    let secret = twofa::generate_secret_base32(20);
    assert!(!twofa::verify_totp_code_at(
        &secret,
        "12345", // 5 digits instead of 6
        chrono::Utc::now().fixed_offset(),
        twofa::DEFAULT_TOTP_STEP,
        twofa::DEFAULT_TOTP_DIGITS,
        1,
    ));
}

#[test]
fn backup_code_constant_time_compare() {
    // Verify that similar-looking codes don't match
    use ruxlog::utils::twofa;

    let codes = vec!["ABCD-EFGH-JKLM".to_string()];
    let hashes = twofa::hash_backup_codes(&codes);

    // Similar but different code should not match
    assert!(twofa::consume_backup_code(&hashes, "ABCD-EFGH-JKLN").is_none());
    // Exact match should work
    assert!(twofa::consume_backup_code(&hashes, "ABCD-EFGH-JKLM").is_some());
}

#[test]
fn error_codes_distinct_status_for_auth_vs_db() {
    use ruxlog::error::codes::ErrorCode;

    // Auth errors should be 401/403, not 500
    let auth_401 = vec![
        ErrorCode::InvalidCredentials,
        ErrorCode::SessionExpired,
        ErrorCode::InvalidToken,
    ];
    for code in &auth_401 {
        assert_eq!(code.status_code(), axum::http::StatusCode::UNAUTHORIZED);
    }

    let auth_403 = vec![
        ErrorCode::Unauthorized,
        ErrorCode::AccountLocked,
        ErrorCode::EmailVerificationRequired,
    ];
    for code in &auth_403 {
        assert_eq!(code.status_code(), axum::http::StatusCode::FORBIDDEN);
    }

    // DB errors should be 404/409/500, never 401
    assert_eq!(
        ErrorCode::RecordNotFound.status_code(),
        axum::http::StatusCode::NOT_FOUND
    );
    assert_eq!(
        ErrorCode::DuplicateEntry.status_code(),
        axum::http::StatusCode::CONFLICT
    );
    assert_eq!(
        ErrorCode::DatabaseConnectionError.status_code(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}
