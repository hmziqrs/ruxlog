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

// --- CSRF Guard Tests (per-session scheme, plan Phase 5) ---
//
// The token is base64(HMAC-SHA256(signing_key, session_id)) and is validated
// statelessly by `csrf_guard`, which reads the request `Session`. The test
// router therefore mounts the SessionManagerLayer (MemoryStore) as the OUTER
// layer so the Session is available to the guard, plus the `/csrf/v1/generate`
// bootstrap route so a real per-session token can be obtained.

fn csrf_router() -> Router {
    use tower_sessions::{MemoryStore, SessionManagerLayer};
    let store = MemoryStore::default();
    Router::new()
        .route("/protected", post(ok_handler))
        .route(
            "/csrf/v1/generate",
            post(ruxlog::modules::csrf_v1::controller::generate),
        )
        // csrf_guard INNER, SessionManagerLayer OUTER (applied last) so the
        // Session is in the request extensions when the guard runs.
        .layer(middleware::from_fn(csrf_guard))
        .layer(SessionManagerLayer::new(store))
}

/// Bootstrap a session via `/csrf/v1/generate`, returning the bound CSRF token
/// and the session cookie (name, value) to carry on the next request.
async fn csrf_bootstrap(app: &Router) -> (String, String, String) {
    use axum::body::to_bytes;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/csrf/v1/generate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("generate sets a session cookie")
        .to_str()
        .unwrap()
        .to_string();
    let pair = set_cookie.split(';').next().unwrap_or(&set_cookie).trim();
    let name = pair.split('=').next().unwrap_or("").to_string();
    let value = pair.split('=').nth(1).unwrap_or("").to_string();

    let bytes = to_bytes(res.into_body(), 4096).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = json["token"].as_str().unwrap().to_string();
    (token, name, value)
}

#[tokio::test]
async fn csrf_rejects_missing_token() {
    let app = csrf_router();
    // POST with neither a session cookie nor a token. A new session has no id
    // yet, so the guard returns MissingToken (401) — a cross-site POST cannot
    // pass.
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
async fn csrf_rejects_token_bound_to_a_different_session() {
    let app = csrf_router();
    // Two independent sessions, each with its own bound token.
    let (token_a, _name_a, _value_a) = csrf_bootstrap(&app).await;
    let (_token_b, name_b, value_b) = csrf_bootstrap(&app).await;

    // Session B's request carrying session A's token → rejected. This is the
    // load-bearing per-session property: a token minted for one session must
    // not validate another.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .header("csrf-token", &token_a)
                .header("cookie", format!("{name_b}={value_b}"))
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
    let (token, name, value) = csrf_bootstrap(&app).await;

    // Reuse the session cookie + the bound token on a mutating request.
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .header("csrf-token", &token)
                .header("cookie", format!("{name}={value}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn csrf_exempt_bootstrap_path_needs_no_token() {
    let app = csrf_router();
    // `/csrf/v1/generate` is exempt (it is the bootstrap) and must succeed with
    // no token at all — it both issues the token and establishes the session.
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/csrf/v1/generate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// --- Input Validation Edge Cases ---

#[test]
fn empty_string_not_accepted_as_totp_code() {
    use ruxlog::utils::twofa;

    let secret = twofa::generate_secret_base32(20).expect("CSPRNG available");
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

    let secret = twofa::generate_secret_base32(20).expect("CSPRNG available");
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

    let secret = twofa::generate_secret_base32(20).expect("CSPRNG available");
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
fn user_json_never_leaks_secret_fields() {
    // The password hash, TOTP seed, and backup-code hashes are all flagged
    // #[serde(skip_serializing)] so that any handler returning a user model
    // (login, 2FA verify/disable, OAuth, register) cannot leak them. This guards
    // against a future field losing that attribute. See plan Phase 2a.
    use ruxlog::db::sea_models::user::{self, UserRole};

    let now = chrono::Utc::now().fixed_offset();
    let model = user::Model {
        id: 1,
        name: "Test".into(),
        email: "test@example.com".into(),
        password: Some("dummy-argon2-hash".into()),
        avatar_id: None,
        is_verified: true,
        role: UserRole::User,
        two_fa_enabled: true,
        two_fa_secret: Some("JBSWY3DPEHPK3PXP".into()),
        two_fa_backup_codes: Some(serde_json::json!(["$argon2id$dummy"])),
        google_id: None,
        oauth_provider: None,
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_value(&model).expect("user model must serialize");
    let obj = json.as_object().expect("serialized user is an object");

    for secret_field in ["password", "two_fa_secret", "two_fa_backup_codes"] {
        assert!(
            !obj.contains_key(secret_field),
            "{secret_field} leaked into serialized user JSON: {json}"
        );
    }

    // Non-secret fields must still be present (sanity check).
    assert_eq!(
        obj.get("email").and_then(|v| v.as_str()),
        Some("test@example.com")
    );
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
