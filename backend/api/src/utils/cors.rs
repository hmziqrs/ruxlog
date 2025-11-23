use std::env;

use axum::http::HeaderValue;

/// Build the list of allowed CORS origins.
///
/// This mirrors the configuration used in `main.rs` so both the CORS layer
/// and our middleware share the same source of truth.
pub fn get_allowed_origins() -> Vec<HeaderValue> {
    let mut default_origins: Vec<String> = vec![
        "http://localhost:8081",
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "http://127.0.0.1:8000",
        "http://127.0.0.1:8888",
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "http://127.0.0.1:3001",
        "http://127.0.0.1:3002",
        "http://127.0.0.1:3333",
        "https://127.0.0.1:3333",
        "http://192.168.0.101:3333",
        "http://192.168.0.101:3000",
        "http://192.168.0.101:8000",
        "http://192.168.0.101:8080",
        "http://192.168.0.101:8888",
        "http://192.168.0.23:3333",
        "http://192.168.0.23:3000",
        "http://192.168.0.23:8080",
        "http://192.168.0.23:8888",
        "https://hzmiqrs.com",
        "https://hmziq.rs",
        "https://blog.hmziq.rs",
    ]
    .into_iter()
    .map(|val| val.to_string())
    .collect();

    if let Ok(admin_port) = env::var("ADMIN_PORT") {
        default_origins.push(format!("http://localhost:{}", admin_port));
        default_origins.push(format!("http://127.0.0.1:{}", admin_port));
    }

    if let Ok(consumer_port) = env::var("CONSUMER_PORT") {
        default_origins.push(format!("http://localhost:{}", consumer_port));
        default_origins.push(format!("http://127.0.0.1:{}", consumer_port));
    }

    if let Ok(env_allowed_origin) = env::var("ALLOWED_ORIGINS") {
        let env_origins: Vec<String> = env_allowed_origin
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        default_origins.extend(env_origins);
    }

    default_origins
        .iter()
        .map(|origin| origin.parse::<HeaderValue>().unwrap())
        .collect()
}
