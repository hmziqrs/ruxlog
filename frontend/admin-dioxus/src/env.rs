pub const APP_API_URL: &str = match std::option_env!("SITE_URL") {
    Some(url) => url,
    None => "http://localhost:8888",
};
