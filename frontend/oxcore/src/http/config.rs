use std::cell::RefCell;

thread_local! {
    static BASE_URL: RefCell<String> = RefCell::new(String::new());
    static CSRF_TOKEN: RefCell<String> = RefCell::new(String::new());
}

/// Configure HTTP client with base URL and CSRF token
/// Call this once at app startup
pub fn configure(base_url: impl Into<String>, csrf_token: impl Into<String>) {
    BASE_URL.with(|url| *url.borrow_mut() = base_url.into());
    CSRF_TOKEN.with(|token| *token.borrow_mut() = csrf_token.into());
}

pub(crate) fn get_base_url() -> String {
    BASE_URL.with(|url| url.borrow().clone())
}

pub(crate) fn get_csrf_token() -> String {
    CSRF_TOKEN.with(|token| token.borrow().clone())
}
