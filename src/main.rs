pub mod db;
pub mod modules;
mod router;
pub mod services;
pub mod state;

use axum_login::AuthManagerLayerBuilder;
use std::env;
use time;

use axum_extra::extract::cookie::SameSite;
use services::{auth::AuthBackend, redis::init_redis_store};
pub use state::AppState;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    tracing::info!("Starting server.");
    let pool = db::connect::get_pool().await;
    tracing::info!("Postgres connection established.");
    let backend = AuthBackend::new(&pool);
    let state = AppState { db_pool: pool };
    let (redis_pool, redis_connection) = init_redis_store().await?;
    tracing::info!("Redis successfully established.");
    let session_store = RedisStore::new(redis_pool);
    // let key = Key::generate();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24)))
        .with_same_site(SameSite::Strict)
        .with_secure(true);
    // .with_private(key);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let app = router::router().layer(auth_layer).with_state(state);

    // run our app with hyper, listening globally on port 3000
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let address = format!("{}:{}", host, port);
    tracing::info!("Listening on http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    redis_connection.await??;

    Ok(())
}
