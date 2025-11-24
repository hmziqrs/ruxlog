pub const APP_API_URL: &str = match std::option_env!("SITE_URL") {
    Some(url) => url,
    None => "http://localhost:1100",
};

pub const APP_CSRF_TOKEN: &str = match std::option_env!("CSRF_KEY") {
    Some(key) => key,
    None => "dev-csrf-key",
};
