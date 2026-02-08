use once_cell::sync::OnceCell;

static BASE_URL: OnceCell<String> = OnceCell::new();
static CSRF_TOKEN: OnceCell<String> = OnceCell::new();

/// Configure HTTP client with base URL and CSRF token
/// Call this once at app startup
pub fn configure(base_url: impl Into<String>, csrf_token: impl Into<String>) {
    let _ = BASE_URL.set(base_url.into());
    let _ = CSRF_TOKEN.set(csrf_token.into());
}

pub(crate) fn get_base_url() -> String {
    BASE_URL.get().cloned().unwrap_or_default()
}

pub(crate) fn get_csrf_token() -> String {
    CSRF_TOKEN.get().cloned().unwrap_or_default()
}
