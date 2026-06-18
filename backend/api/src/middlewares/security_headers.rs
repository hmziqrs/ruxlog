//! Security headers middleware.
//!
//! Adds standard security headers to all responses:
//! - X-Content-Type-Options: nosniff
//! - X-Frame-Options: DENY
//! - Referrer-Policy: strict-origin-when-cross-origin
//! - Permissions-Policy: camera=(), microphone=(), geolocation=()
//! - X-XSS-Protection: 0 (deprecated but some scanners expect it)
//! - Strict-Transport-Security: max-age=31536000; includeSubDomains (HSTS)
//! - Content-Security-Policy: restrictive allowlist (see plan Phase 6c)
//!
//! HSTS and CSP are env-overridable for deployment flexibility, but default to
//! secure values. Setting `HSTS_HEADER=""` (empty) omits HSTS (e.g. for local
//! HTTP-only dev); `CONTENT_SECURITY_POLICY` replaces the default policy.

use axum::{
    extract::Request,
    http::{header, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

const NOSNIFF: &str = "nosniff";
const DENY: &str = "DENY";
const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";
const PERMISSIONS_POLICY: &str = "camera=(), microphone=(), geolocation=()";
const XSS_PROTECTION: &str = "0";

// HSTS: ≥1 year, cover subdomains. Browsers ignore this over plain HTTP and
// exempt localhost, so emitting it unconditionally is safe-by-default.
const DEFAULT_HSTS: &str = "max-age=31536000; includeSubDomains";

// Restrictive default CSP for THIS API's responses. Caveat: the middleware
// only sets headers on the JSON API origin; the Dioxus WASM frontends are
// served from separate origins and must carry their own CSP — so `script-src
// 'self'` here is NOT the primary stored-XSS control. The primary control is
// the server-side ammonia sanitizer (utils/sanitize.rs, plan Phase 6e) that
// strips `<script>`/event-handler attributes/`javascript:` from post content
// on read. These headers remain defence-in-depth for any HTML the API emits.
// Inline styles + Google Fonts are permitted (the apps inject styles / font).
const DEFAULT_CSP: &str = "default-src 'self'; \
    script-src 'self'; \
    style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
    font-src 'self' https://fonts.gstatic.com; \
    img-src 'self' data: https:; \
    media-src 'self'; \
    connect-src 'self'; \
    object-src 'none'; \
    base-uri 'self'; \
    frame-ancestors 'none'; \
    form-action 'self'";

/// Resolve the HSTS header value, honouring the `HSTS_HEADER` env override.
/// An explicitly empty value omits the header (opt-out for local dev).
fn hsts_header_value() -> Option<HeaderValue> {
    let raw = std::env::var("HSTS_HEADER").unwrap_or_else(|_| DEFAULT_HSTS.to_string());
    if raw.is_empty() {
        return None;
    }
    HeaderValue::from_str(&raw).ok()
}

/// Resolve the CSP header value, honouring the `CONTENT_SECURITY_POLICY` env
/// override.
fn csp_header_value() -> Option<HeaderValue> {
    let raw =
        std::env::var("CONTENT_SECURITY_POLICY").unwrap_or_else(|_| DEFAULT_CSP.to_string());
    if raw.is_empty() {
        return None;
    }
    HeaderValue::from_str(&raw).ok()
}

pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static(NOSNIFF),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static(DENY));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static(REFERRER_POLICY),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(PERMISSIONS_POLICY),
    );
    // X-XSS-Protection is deprecated but some security scanners still flag its absence
    headers.insert("x-xss-protection", HeaderValue::from_static(XSS_PROTECTION));

    if let Some(hsts) = hsts_header_value() {
        headers.insert(header::STRICT_TRANSPORT_SECURITY, hsts);
    }
    if let Some(csp) = csp_header_value() {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn ok() -> &'static str {
        "ok"
    }

    async fn run_headers() -> axum::http::HeaderMap {
        let app = axum::Router::new()
            .route("/", axum::routing::get(ok))
            .layer(axum::middleware::from_fn(security_headers));
        let res = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        res.headers().clone()
    }

    // Default policy sanity (no env mutation → no cross-test races). The
    // HSTS_HEADER / CONTENT_SECURITY_POLICY override logic is env-reading only
    // and verified by deployment; we assert the secure defaults ship here.
    #[tokio::test]
    async fn hsts_and_csp_present_with_secure_defaults() {
        let h = run_headers().await;
        let hsts = h
            .get(header::STRICT_TRANSPORT_SECURITY)
            .expect("HSTS header present")
            .to_str()
            .unwrap();
        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
        let csp = h
            .get(header::CONTENT_SECURITY_POLICY)
            .expect("CSP header present")
            .to_str()
            .unwrap();
        // script-src restricted to 'self' only (blocks inline <script> from
        // stored XSS via dangerous_inner_html) and object-src disabled.
        assert!(
            csp.contains("script-src 'self';"),
            "script-src must be 'self' only, got: {csp}"
        );
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }
}
