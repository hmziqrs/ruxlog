use axum::{
    http::{HeaderName, HeaderValue},
    middleware, routing,
};
use axum_client_ip::ClientIpSource;
use axum_extra::extract::cookie::SameSite;
use axum_login::AuthManagerLayerBuilder;
use std::{env, net::SocketAddr, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
};
use tower_sessions::{cookie::Key, Expiry, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;

use modules::csrf_v1;
use ruxlog::{
    db, middlewares, modules, router,
    services::{self, auth::AuthBackend, redis::init_redis_store},
    state::{AppState, ObjectStorageConfig, OptimizerConfig},
    utils::telemetry,
};

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

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => None,
            }
        })
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_u8(key: &str, default: u8) -> u8 {
    let candidate = env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u8>().ok())
        .unwrap_or(default);
    candidate.clamp(0, 100)
}

fn env_with_fallback(keys: &[&str], default: Option<&str>) -> Option<String> {
    for key in keys {
        if let Ok(value) = env::var(key) {
            if !value.trim().is_empty() {
                return Some(value);
            }
        }
    }

    default.map(|value| value.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let _telemetry_guard = telemetry::init();

    telemetry::init_pool_metrics();

    let cookie_key_str = env::var("COOKIE_KEY").expect("COOKIE_KEY must be set");

    let sea_db = db::sea_connect::get_sea_connection().await;

    let backend = AuthBackend::new(&sea_db);
    let (redis_pool, redis_connection) = init_redis_store().await?;
    let mailer = services::mail::smtp::create_connection().await;

    let bucket = env_with_fallback(&["S3_BUCKET", "AWS_S3_BUCKET"], None)
        .expect("S3_BUCKET or AWS_S3_BUCKET must be set");
    let access_key = env_with_fallback(&["S3_ACCESS_KEY", "AWS_ACCESS_KEY_ID"], None)
        .expect("S3_ACCESS_KEY or AWS_ACCESS_KEY_ID must be set");
    let secret_key = env_with_fallback(&["S3_SECRET_KEY", "AWS_SECRET_ACCESS_KEY"], None)
        .expect("S3_SECRET_KEY or AWS_SECRET_ACCESS_KEY must be set");
    let endpoint = env_with_fallback(&["S3_ENDPOINT", "AWS_ENDPOINT", "GARAGE_S3_ENDPOINT"], None)
        .expect("S3_ENDPOINT, AWS_ENDPOINT, or GARAGE_S3_ENDPOINT must be set");
    let public_url = env_with_fallback(&["S3_PUBLIC_URL", "AWS_S3_PUBLIC_URL"], None)
        .unwrap_or_else(|| {
            // Fall back to direct endpoint when explicit public URL is missing.
            endpoint.clone()
        });

    let object_storage = ObjectStorageConfig {
        region: env_with_fallback(
            &[
                "S3_REGION",
                "GARAGE_S3_REGION",
                "AWS_S3_REGION",
                "AWS_REGION",
            ],
            Some("auto"),
        )
        .unwrap(),
        account_id: env::var("S3_ACCOUNT_ID").unwrap_or_else(|_| "local".to_string()),
        bucket,
        access_key,
        secret_key,
        public_url,
        endpoint,
    };

    println!("Object Storage Config: {:?}", object_storage);

    let s3_config = aws_config::from_env()
        .endpoint_url(&object_storage.endpoint)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &object_storage.access_key,
            &object_storage.secret_key,
            None,
            None,
            "S3Compatible",
        ))
        .region(aws_sdk_s3::config::Region::new(
            object_storage.region.clone(),
        ))
        .load()
        .await;

    let s3_client = aws_sdk_s3::Client::new(&s3_config);

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

    let optimizer = OptimizerConfig {
        enabled: env_bool("OPTIMIZE_ON_UPLOAD", true),
        max_pixels: env_u64("OPTIMIZER_MAX_PIXELS", 40_000_000),
        keep_original: env_bool("OPTIMIZER_KEEP_ORIGINAL", true),
        default_webp_quality: env_u8("OPTIMIZER_WEBP_QUALITY_DEFAULT", 80),
    };

    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let supabase_key =
        env::var("SUPABASE_SERVICE_ROLE_KEY").expect("SUPABASE_SERVICE_ROLE_KEY must be set");
    let supabase = services::supabase::SupabaseClient::new(supabase_url, supabase_key);

    let state = AppState {
        sea_db,
        redis_pool: redis_pool.clone(),
        mailer,
        object_storage,
        s3_client,
        optimizer,
        meter: telemetry::global_meter(),
        supabase,
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

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let ip_source: ClientIpSource = env::var("IP_SOURCE")
        .unwrap_or_else(|_| "ConnectInfo".to_string())
        .parse()
        .expect("Invalid IP_SOURCE value");

    let app = router::router()
        .layer(ip_source.into_extension())
        .layer(auth_layer)
        //     config: governor_conf,
        // })
        .layer(compression)
        .layer(middleware::from_fn(
            middlewares::http_metrics::track_metrics,
        ))
        .layer(middleware::from_fn(
            middlewares::request_id::request_id_middleware,
        ))
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
