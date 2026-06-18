use once_cell::sync::OnceCell;
use std::sync::Mutex;

/// Backend base URL. Set once at startup; never changes.
static BASE_URL: OnceCell<String> = OnceCell::new();

/// Per-session CSRF token. Replaceable because it is bound to the session id:
/// the backend issues a new token whenever the session changes (see
/// `refresh_csrf_token`). The previous design baked a constant in at build time,
/// which the per-session HMAC scheme (plan Phase 5) rejects — so this is now a
/// mutable store populated from `/csrf/v1/generate`.
///
/// NOTE (audit F#16 — intentionally accepted): the token has **login-session
/// granularity** — it is HMAC-bound to the session id and rotated when the
/// session id changes (login, logout). It is NOT re-rotated at intra-session
/// trust transitions (2FA enable/disable, password change, role elevation)
/// because the session id does not change at those points. A token captured
/// before such a transition therefore stays valid after it. There is no
/// CSRF-side "step-up refresh." This is accepted: it follows directly from the
/// locked "leave the login flow as-is" decision (no 2FA gate at login ⇒ no
/// 2FA-completion trust transition to protect), and CSRF's job is to bind to
/// the authenticated session, not to model step-up freshness. Revisit only if a
/// login-time 2FA challenge is introduced. See `docs/CRYPTO_AUDIT.md` →
/// "Accepted Deferrals".
static CSRF_TOKEN: Mutex<String> = Mutex::new(String::new());

/// Configure the HTTP client base URL. Call once at app startup.
pub fn configure(base_url: impl Into<String>) {
    let _ = BASE_URL.set(base_url.into());
}

/// Store a freshly-issued CSRF token. Called after `/csrf/v1/generate`.
pub fn set_csrf_token(token: impl Into<String>) {
    if let Ok(mut guard) = CSRF_TOKEN.lock() {
        *guard = token.into();
    }
}

pub(crate) fn get_base_url() -> String {
    BASE_URL.get().cloned().unwrap_or_default()
}

/// Current CSRF token (attached to every mutating request). Returns an empty
/// string until the first successful `/csrf/v1/generate` round-trip completes.
pub(crate) fn get_csrf_token() -> String {
    CSRF_TOKEN.lock().map(|g| g.clone()).unwrap_or_default()
}
