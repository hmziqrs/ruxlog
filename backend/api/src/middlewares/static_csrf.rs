//! Per-session CSRF token (stateless double-submit, plan Phase 5).
//!
//! The token is `base64(HMAC-SHA256(signing_key, session_id))`, where
//! `signing_key` is HKDF-SHA256-derived from `COOKIE_KEY` under the
//! `b"ruxlog-csrf-v1"` label — domain-separated from the cookie private key,
//! which derives from the same input under its own label. Because the token is
//! bound to the *session id*, a token minted for one session cannot validate for
//! another — defeating the previous scheme, which signed nothing and issued the
//! same constant token to every client.
//!
//! Verification is stateless: the middleware recomputes the HMAC from the
//! request's session id and constant-time-compares it to the `csrf-token`
//! header. No Redis lookup is needed.
//!
//! Bootstrap: `/csrf/v1/generate` is exempt and both *issues* the token and
//! *materializes* the session (so the client receives a session cookie in the
//! same response). The client then attaches that token to every mutating
//! request. Session rotation (e.g. on login) changes the session id, which
//! invalidates the prior token — the client re-fetches from `/csrf/v1/generate`.

use std::sync::OnceLock;

use axum::{extract::Request, middleware::Next, response::Response};
use base64::prelude::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tower_sessions::Session;
use tracing::{debug, instrument, warn};

use crate::error::CsrfError;

type HmacSha256 = Hmac<Sha256>;

static CSRF_SIGNING_KEY: OnceLock<Vec<u8>> = OnceLock::new();

/// HKDF-SHA256 (RFC 5869) extracting+expanding `ikm` under `info`, yielding a
/// 32-byte key. Single-block expand is sufficient for a 32-byte output.
///
/// Used to domain-separate the CSRF signing key from the cookie private key:
/// both derive from the same `COOKIE_KEY` input, but each uses a distinct
/// `info` label, so the two output keys are cryptographically independent.
fn hkdf_sha256(ikm: &[u8], info: &[u8]) -> [u8; 32] {
    // Extract: PRK = HMAC-SHA256(salt, IKM); salt absent ⇒ HashLen (32) zero
    // bytes (RFC 5869 §2.2).
    let mut extract = HmacSha256::new_from_slice(&[0u8; 32]).expect("HMAC accepts key");
    extract.update(ikm);
    let prk = extract.finalize().into_bytes();

    // Expand: for L ≤ 32, OKM = T(1) = HMAC-SHA256(PRK, info ‖ 0x01).
    let mut expand = HmacSha256::new_from_slice(&prk).expect("HMAC accepts key");
    expand.update(info);
    expand.update(&[0x01]);
    let okm = expand.finalize().into_bytes();

    let mut out = [0u8; 32];
    out.copy_from_slice(&okm);
    out
}

/// The per-deployment HMAC key for CSRF tokens. HKDF-derived from `COOKIE_KEY`
/// under the domain-separation label `b"ruxlog-csrf-v1"`, so it is
/// cryptographically independent of the cookie private key (which
/// `cookie::Key::derive_from` derives from the same input under its own label).
/// No separate secret must be managed. Cached for the process lifetime.
fn csrf_signing_key() -> &'static [u8] {
    CSRF_SIGNING_KEY.get_or_init(|| {
        // Fail-closed: a CSRF signing key must never be derived from a baked
        // constant, since that would silently issue forgeable tokens to every
        // client. In production COOKIE_KEY must be set (>= 32 bytes); otherwise
        // we panic rather than degrade to an insecure default. Tests do not
        // configure the environment, so they get a fixed deterministic key.
        let ikm = match std::env::var("COOKIE_KEY") {
            Ok(k) if !k.is_empty() => k.into_bytes(),
            #[cfg(not(test))]
            _ => panic!(
                "COOKIE_KEY must be set (>= 32 bytes) to derive the CSRF signing key; \
                 see CRYPTO_AUDIT.md Part V V-HIGH-6"
            ),
            #[cfg(test)]
            _ => b"ruxlog-test-csrf-key-deterministic".to_vec(),
        };
        hkdf_sha256(&ikm, b"ruxlog-csrf-v1").to_vec()
    })
}

/// `base64(HMAC-SHA256(signing_key, session_id))` — the per-session CSRF token.
/// Pure and shared by `generate` (issue) and `csrf_guard` (verify), guaranteeing
/// both sides use one algorithm.
pub(crate) fn compute_csrf_token(session_id: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(csrf_signing_key()).expect("HMAC accepts any key length");
    mac.update(session_id.as_bytes());
    BASE64_STANDARD.encode(mac.finalize().into_bytes())
}

/// Exact-match CSRF exemptions. Compared by full path / segment — never
/// `starts_with` — so a prefix-cousin (e.g. `/billing/v1/webhook-evil`) cannot
/// slip past the guard.
fn is_csrf_exempt(path: &str) -> bool {
    if matches!(
        path,
        "/auth/google/v1/callback" | "/auth/google/v1/login" | "/csrf/v1/generate"
    ) {
        return true;
    }
    // Webhook receivers: exactly /billing/v1/webhook/{provider} (5 segments
    // when split on '/': ["", "billing", "v1", "webhook", "<provider>"]).
    let mut segs = path.split('/');
    let _leading = segs.next();
    matches!(
        (
            segs.next(),
            segs.next(),
            segs.next(),
            segs.next(),
            segs.next(),
        ),
        (Some("billing"), Some("v1"), Some("webhook"), Some(_), None)
    )
}

#[instrument(skip(session, req, next), fields(token_present, result, path))]
pub async fn csrf_guard(session: Session, req: Request, next: Next) -> Result<Response, CsrfError> {
    let path = req.uri().path();
    tracing::Span::current().record("path", path);

    // Safe (read-only) methods never require CSRF protection.
    if matches!(
        *req.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        tracing::Span::current().record("result", "safe_method_exempted");
        return Ok(next.run(req).await);
    }

    if is_csrf_exempt(path) {
        debug!("Skipping CSRF check for exempted route: {}", path);
        tracing::Span::current().record("result", "exempted");
        return Ok(next.run(req).await);
    }

    // Resolve the session id. No session ⇒ no valid token is possible, so deny:
    // the client must obtain a token from /csrf/v1/generate first.
    let expected = match session.id() {
        Some(id) => compute_csrf_token(&id.to_string()),
        None => {
            warn!("CSRF check failed: request has no session id");
            tracing::Span::current().record("result", "no_session");
            return Err(CsrfError::MissingToken);
        }
    };

    let Some(token) = req.headers().get("csrf-token") else {
        warn!("CSRF token missing from request");
        tracing::Span::current().record("token_present", false);
        tracing::Span::current().record("result", "missing");
        return Err(CsrfError::MissingToken);
    };
    tracing::Span::current().record("token_present", true);

    let provided = match token.to_str() {
        Ok(s) => s.as_bytes(),
        Err(_) => {
            warn!("CSRF token header not valid string");
            tracing::Span::current().record("result", "invalid_header");
            return Err(CsrfError::InvalidHeader);
        }
    };

    // Constant-time comparison. The expected length is fixed by the HMAC output,
    // so a length check leaks nothing; differing lengths simply cannot match.
    let ok = provided.len() == expected.len() && bool::from(provided.ct_eq(expected.as_bytes()));
    if ok {
        debug!("CSRF token validated successfully");
        tracing::Span::current().record("result", "valid");
        Ok(next.run(req).await)
    } else {
        warn!("CSRF token mismatch");
        tracing::Span::current().record("result", "token_mismatch");
        Err(CsrfError::TokenMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_stable_for_a_session_and_differs_across_sessions() {
        // Same session id → same token (idempotent issuance).
        let a1 = compute_csrf_token("session-A");
        let a2 = compute_csrf_token("session-A");
        assert_eq!(a1, a2);
        assert!(!a1.is_empty());

        // Different session id → different token. This is the load-bearing
        // property: a token minted for session A must not validate session B.
        let b = compute_csrf_token("session-B");
        assert_ne!(a1, b);
    }

    #[test]
    fn hkdf_label_gives_domain_separation() {
        // Same input material, different info labels ⇒ independent keys. This
        // is why the CSRF signing key and the cookie private key do not collide
        // even though both derive from COOKIE_KEY.
        let ikm = b"same-secret-cookie-key";
        let csrf = hkdf_sha256(ikm, b"ruxlog-csrf-v1");
        let cookie = hkdf_sha256(ikm, b"ruxlog-cookie-private-key");
        assert_ne!(
            csrf, cookie,
            "distinct info labels must yield distinct keys"
        );

        // Deterministic: the same (ikm, info) pair reproduces the same key.
        assert_eq!(csrf, hkdf_sha256(ikm, b"ruxlog-csrf-v1"));
    }

    #[test]
    fn exempt_list_is_exact_not_prefix() {
        // The genuine webhook receiver is exempt …
        assert!(is_csrf_exempt("/billing/v1/webhook/stripe"));
        assert!(is_csrf_exempt("/billing/v1/webhook/paddle"));
        // … but a prefix-cousin must NOT be (prevents bypass).
        assert!(!is_csrf_exempt("/billing/v1/webhook-evil/x"));
        assert!(!is_csrf_exempt("/billing/v1/webhook")); // no provider segment
        assert!(!is_csrf_exempt("/billing/v1/webhook/a/b")); // too many segments

        // OAuth + generate bootstrap are exempt (exact).
        assert!(is_csrf_exempt("/auth/google/v1/callback"));
        assert!(is_csrf_exempt("/auth/google/v1/login"));
        assert!(is_csrf_exempt("/csrf/v1/generate"));
        // … but cousins are not.
        assert!(!is_csrf_exempt("/auth/google/v1/callback-evil"));
        assert!(!is_csrf_exempt("/csrf/v1/generateX"));
    }
}

/// End-to-end middleware tests: build a real Router with the SessionManagerLayer
/// (MemoryStore, applied OUTER so the Session is visible to `csrf_guard`) and
/// exercise the full request flow.
#[cfg(test)]
mod middleware_tests {
    use super::*;
    use crate::modules::csrf_v1;
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request, StatusCode},
        middleware,
        routing::{get, post},
        Router,
    };
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, SessionManagerLayer};

    async fn ok() -> &'static str {
        "ok"
    }

    fn test_app() -> Router {
        let store = MemoryStore::default();
        Router::new()
            .route("/mutate", post(ok))
            .route("/read", get(ok))
            .route("/csrf/v1/generate", post(csrf_v1::controller::generate))
            // csrf_guard INNER, SessionManagerLayer OUTER (applied last) → the
            // Session is in the request extensions when csrf_guard runs.
            .layer(middleware::from_fn(csrf_guard))
            .layer(SessionManagerLayer::new(store))
    }

    /// Pull the `name=value` cookie pair out of a `Set-Cookie` header value
    /// (ignoring attributes like `; Path=/`).
    fn cookie_pair(set_cookie: &str) -> &str {
        set_cookie.split(';').next().unwrap_or(set_cookie).trim()
    }

    /// Call `/csrf/v1/generate` against the app, returning the bound CSRF token
    /// and the session cookie name+value (to carry on the next request).
    async fn bootstrap(
        app: &Router,
        seed_cookie: Option<(&str, &str)>,
    ) -> (String, String, String) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri("/csrf/v1/generate")
            .body(Body::empty())
            .unwrap();
        if let Some((name, value)) = seed_cookie {
            req.headers_mut()
                .insert("cookie", format!("{name}={value}").parse().unwrap());
        }
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let set_cookie = res
            .headers()
            .get("set-cookie")
            .expect("generate sets a session cookie")
            .to_str()
            .unwrap()
            .to_string();
        let pair = cookie_pair(&set_cookie).to_string();
        let name = pair.split('=').next().unwrap_or("").to_string();
        let value = pair.split('=').nth(1).unwrap_or("").to_string();

        let bytes = to_bytes(res.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let token = json["token"].as_str().unwrap().to_string();
        (token, name, value)
    }

    #[tokio::test]
    async fn mutating_request_without_header_is_rejected() {
        let app = test_app();
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mutate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // No session → MissingToken → 401. A cross-site form POST cannot pass.
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn safe_methods_are_not_checked() {
        let app = test_app();
        // GET without any token or session must succeed — safe methods are exempt.
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/read")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_token_with_matching_session_is_accepted() {
        let app = test_app();
        let (token, name, value) = bootstrap(&app, None).await;

        // Reuse the session cookie + the bound token on a mutating request.
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mutate")
                    .header("csrf-token", &token)
                    .header("cookie", format!("{name}={value}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn token_is_bound_to_its_session_only() {
        let app = test_app();
        // Two independent sessions, each with its own bound token.
        let (token_a, _name_a, value_a) = bootstrap(&app, None).await;
        let (token_b, name_b, value_b) = bootstrap(&app, None).await;
        assert_ne!(value_a, value_b, "sessions must differ");
        assert_ne!(token_a, token_b, "tokens must differ across sessions");

        // Session B's request carrying session A's token → rejected.
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mutate")
                    .header("csrf-token", &token_a)
                    .header("cookie", format!("{name_b}={value_b}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "a token minted for session A must not validate session B"
        );

        // Session B with its OWN token → accepted (sanity).
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mutate")
                    .header("csrf-token", &token_b)
                    .header("cookie", format!("{name_b}={value_b}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
