//! CSRF token helpers for tests (per-session scheme, plan Phase 5).
//!
//! The CSRF token is `base64(HMAC-SHA256(csrf_signing_key, session_id))`, bound
//! to a specific session. Tests that exercise mutating routes through the real
//! middleware stack must therefore mint a token for the *same* session id the
//! request will carry. Typically a test first calls `POST /csrf/v1/generate`
//! (which both bootstraps the session and returns the bound token), reuses the
//! resulting session cookie, and attaches the returned token to subsequent
//! mutating requests.

use crate::middlewares::static_csrf::compute_csrf_token;

/// Returns a CSRF token valid for the given session id against the per-session
/// CSRF middleware.
pub fn csrf_token_for_session(session_id: &str) -> String {
    compute_csrf_token(session_id)
}

/// Returns the header name and value for the given session id.
/// Usage: `request.header(csrf_header_for_session(sid).0, csrf_header_for_session(sid).1)`
pub fn csrf_header_for_session(session_id: &str) -> (&'static str, String) {
    ("csrf-token", csrf_token_for_session(session_id))
}
