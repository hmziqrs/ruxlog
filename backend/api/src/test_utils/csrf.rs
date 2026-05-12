//! CSRF token helpers for tests.

use base64::Engine;

/// Default CSRF key used in dev/test environments.
pub const TEST_CSRF_KEY: &str = "ultra-instinct-goku";

/// Returns a base64-encoded CSRF token valid against the static CSRF middleware.
pub fn test_csrf_token() -> String {
    base64::engine::general_purpose::STANDARD.encode(TEST_CSRF_KEY)
}

/// Returns the header name and value for CSRF.
/// Usage: `request.header(csrf_header().0, csrf_header().1)`
pub fn csrf_header() -> (&'static str, String) {
    ("csrf-token", test_csrf_token())
}
