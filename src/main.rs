pub mod constants;
pub mod db;
pub mod middlewares;
pub mod modules;
mod router;
pub mod services;
pub mod state;

use axum::{
    http::{HeaderValue, StatusCode},
    middleware,
    response::IntoResponse,
    routing, Json,
};
use axum_client_ip::SecureClientIpSource;
use axum_login::AuthManagerLayerBuilder;
use modules::csrf_v1;
use serde_json::json;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use time;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer};

// use axum_csrf::{CsrfConfig, CsrfLayer, Key as CsrfKey};
use axum_extra::extract::cookie::SameSite;
use services::{auth::AuthBackend, redis::init_redis_store};
pub use state::AppState;
use tower_sessions::{cookie::Key, Expiry, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;

fn hex_to_512bit_key(hex: &str) -> [u8; 64] {
    use sha2::{Digest, Sha512};
    let bytes = hex::decode(hex).expect("Invalid hex string");
    let mut hasher = Sha512::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut array = [0u8; 64];
    array.copy_from_slice(&result);
    array
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    let cookie_key_str = env::var("COOKIE_KEY").expect("COOKIE_KEY must be set");
    // let csrf_key_str = env::var("CSRF_KEY").expect("CSRF_KEY must be set");

    tracing::info!("Starting server.");
    let pool = db::connect::get_pool().await;
    tracing::info!("Postgres connection established.");
    let backend = AuthBackend::new(&pool);
    let (redis_pool, redis_connection) = init_redis_store().await?;
    let mailer = services::mail::smtp::create_connection().await;
    let state = AppState {
        db_pool: pool,
        redis_pool: redis_pool.clone(),
        mailer,
    };

    tracing::info!("Redis successfully established.");
    let session_store = RedisStore::new(redis_pool);
    let cookie_key_byes = hex_to_512bit_key(&cookie_key_str);
    let cookie_key = Key::from(&cookie_key_byes);
    // let csrf_key_byes = hex_to_512bit_key(&csrf_key_str);
    // let csrf_key = CsrfKey::from(&csrf_key_byes);
    // let csrf_config = CsrfConfig::default()
    //     .with_key(Some(csrf_key))
    //     .with_secure(true)
    //     .with_cookie_same_site(SameSite::Strict);
    // .with_cookie_domain(Some("127.0.0.1"));

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24)))
        .with_same_site(SameSite::Strict)
        .with_secure(true)
        .with_private(cookie_key);
    let compression = CompressionLayer::new();
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>()?)
        // .allow_credentials(true)
        // .allow_origin("http://localhost:8000".parse::<HeaderValue>()?)
        .max_age(Duration::from_secs(3600));
    let request_size = RequestBodyLimitLayer::new(1024 * 512);
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(4)
            .burst_size(25)
            .error_handler(|e| {
                let status = StatusCode::TOO_MANY_REQUESTS;
                let body = e.to_string();
                (
                    status,
                    Json(json!({"error": "Too many requests", "message": body})),
                )
                    .into_response()
            })
            .finish()
            .unwrap(),
    );
    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    // a separate background task to clean up
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        tracing::info!("rate limiting storage size: {}", governor_limiter.len());
        governor_limiter.retain_recent();
    });

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = router::router()
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .layer(auth_layer)
        .layer(GovernorLayer {
            config: governor_conf,
        })
        .layer(compression)
        .layer(cors)
        .layer(request_size)
        .layer(middleware::from_fn(middlewares::static_csrf::csrf_gaurd))
        .route(
            "/csrf/v1/generate",
            routing::post(csrf_v1::controller::generate),
        )
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let address = format!("{}:{}", host, port);
    let address = address.parse::<std::net::SocketAddr>()?;
    tracing::info!("Listening on http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    redis_connection.await??;

    Ok(())
}
