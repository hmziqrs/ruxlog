pub const APP_API_URL: &str = match std::option_env!("SITE_URL") {
    Some(val) => val,
    None => "http://localhost:9999",
};
pub const APP_CSRF_TOKEN: &str = match std::option_env!("CSRF_KEY") {
    Some(val) => val,
    None => "dWx0cmEtaW5zdGluY3QtZ29rdQ==",
};

