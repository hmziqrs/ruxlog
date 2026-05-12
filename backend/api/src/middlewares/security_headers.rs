//! Security headers middleware.
//!
//! Adds standard security headers to all responses:
//! - X-Content-Type-Options: nosniff
//! - X-Frame-Options: DENY
//! - Referrer-Policy: strict-origin-when-cross-origin
//! - Permissions-Policy: camera=(), microphone=(), geolocation=()
//! - X-XSS-Protection: 0 (deprecated but some scanners expect it)

use axum::{
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
    extract::Request,
};

const NOSNIFF: &str = "nosniff";
const DENY: &str = "DENY";
const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";
const PERMISSIONS_POLICY: &str = "camera=(), microphone=(), geolocation=()";
const XSS_PROTECTION: &str = "0";

pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static(NOSNIFF),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static(DENY),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static(REFERRER_POLICY),
    );
    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(PERMISSIONS_POLICY),
    );
    // X-XSS-Protection is deprecated but some security scanners still flag its absence
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static(XSS_PROTECTION),
    );

    response
}
