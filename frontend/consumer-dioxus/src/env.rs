pub fn app_api_url() -> String {
    std::env::var("SITE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

pub fn app_csrf_token() -> String {
    std::env::var("CSRF_KEY").unwrap_or_else(|_| "default-csrf-token".to_string())
}
