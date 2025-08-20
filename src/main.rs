pub mod db;
pub mod error;
pub mod extractors;
pub mod middlewares;
pub mod modules;
mod router;
pub mod services;
pub mod state;
pub mod utils;

use axum::{
    http::{HeaderName, HeaderValue},
    middleware, routing,
};
use axum_client_ip::ClientIpSource;
use axum_login::AuthManagerLayerBuilder;
use modules::csrf_v1;
use std::{env, net::SocketAddr, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

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

fn get_allowed_origins() -> Vec<HeaderValue> {
    let mut default_origins: Vec<String> = vec![
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

#[derive(serde::Deserialize)]
struct IpConfig {
    ip_source: ClientIpSource,
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

    let sea_db = db::sea_connect::get_sea_connection().await;

    let backend = AuthBackend::new(&sea_db);
    let (redis_pool, redis_connection) = init_redis_store().await?;
    let mailer = services::mail::smtp::create_connection().await;

    let mut r2 = state::R2Config {
        region: env::var("R2_REGION").unwrap_or_else(|_| "auto".to_string()),
        account_id: env::var("R2_ACCOUNT_ID").expect("R2_ACCOUNT_ID must be set"),
        bucket: env::var("R2_BUCKET").expect("R2_BUCKET must be set"),
        access_key: env::var("R2_ACCESS_KEY").expect("R2_ACCESS_KEY must be set"),
        secret_key: env::var("R2_SECRET_KEY").expect("R2_SECRET_KEY must be set"),
        public_url: env::var("R2_PUBLIC_URL").expect("R2_PUBLIC_URL must be set"),
    };

    let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", &r2.account_id);

    r2.public_url = "https://pub-63743cad4ace41b5903015b89d79fb27.r2.dev".to_string();
    // r2.public_url = format!(
    //     "https://{}.r2.cloudflarestorage.com/{}",
    //     &r2.secret_key, &r2.bucket

    println!("R2 Config: {:?}", r2);

    let r2_config = aws_config::from_env()
        .endpoint_url(endpoint_url)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &r2.access_key,
            &r2.secret_key,
            None,
            None,
            "R2",
        ))
        .region("auto")
        .load()
        .await;

    let s3_client = aws_sdk_s3::Client::new(&r2_config);

    let list_buckets_output = s3_client.list_buckets().send().await?;

    println!("Buckets:");
    for bucket in list_buckets_output.buckets() {
        println!(
            "  - {}: {}",
            bucket.name().unwrap_or_default(),
            bucket.creation_date().map_or_else(
                || "Unknown creation date".to_string(),
                |date| date
                    .fmt(aws_sdk_s3::primitives::DateTimeFormat::DateTime)
                    .unwrap()
            )
        );
    }

    let state = AppState {
        sea_db,
        redis_pool: redis_pool.clone(),
        mailer,
        r2,
        s3_client,
    };

    tracing::info!("Redis successfully established.");
    let session_store = RedisStore::new(redis_pool);
    let cookie_key_byes = hex_to_512bit_key(&cookie_key_str);
    let cookie_key = Key::from(&cookie_key_byes);


    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24 * 14)))
        .with_same_site(SameSite::Lax)
        .with_secure(false)
        .with_http_only(false)
        .with_private(cookie_key);

    let compression = CompressionLayer::new();
    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(vec![
            HeaderName::from_static("csrf-token"),
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT_ENCODING,
            axum::http::header::CONTENT_ENCODING,
        ])
        .expose_headers(vec![
            axum::http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            axum::http::header::SET_COOKIE,
        ])
        .allow_origin(AllowOrigin::list(get_allowed_origins()))
        .allow_credentials(true)
        .max_age(Duration::from_secs(360));
    let request_size = RequestBodyLimitLayer::new(1024 * 512);
    //             (
    //                 status,
    //             )
    //         })
    // a separate background task to clean up

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let ip_config: IpConfig = envy::from_env().unwrap();

    let app = router::router()
        .layer(ip_config.ip_source.into_extension())
        .layer(auth_layer)
        //     config: governor_conf,
        // })
        .layer(compression)
        .layer(request_size)
        .layer(middleware::from_fn(middlewares::static_csrf::csrf_guard))
        .route(
            "/csrf/v1/generate",
            routing::post(csrf_v1::controller::generate),
        )
        .layer(cors)
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
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
