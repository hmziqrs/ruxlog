use std::{env, time::Duration};

use tower_sessions_redis_store::fred::prelude::*;

use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn};

// Minimal Redis configuration.
#[instrument(skip_all)]
fn redis_config() -> Config {
    let host = env::var("REDIS_HOST").expect("REDIS_HOST must be set");
    let port = env::var("REDIS_PORT")
        .expect("REDIS_PORT must be set")
        .parse()
        .expect("REDIS_PORT must be a valid u16");

    info!(redis_host = %host, redis_port = port, "Configuring Redis connection");

    Config {
        username: Some(env::var("REDIS_USER").expect("REDIS_USER must be set")),
        password: Some(env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set")),
        server: ServerConfig::Centralized {
            server: Server::new(host, port),
        },
        ..Default::default()
    }
}

/// Setup the Redis connection pool.
#[instrument(name = "redis_pool_init")]
pub async fn init_redis_store() -> Result<(Pool, JoinHandle<Result<(), Error>>), Error> {
    info!("Initializing Redis connection pool");
    let config = redis_config();
    let connection_config = ConnectionConfig {
        reconnect_on_auth_error: true,
        connection_timeout: Duration::from_millis(1500),
        ..ConnectionConfig::default()
    };

    let re_connection_policy = ReconnectPolicy::new_linear(30, 1000 * 600, 500);

    info!(
        pool_size = 6,
        reconnect_attempts = 30,
        max_reconnect_delay_ms = 600000,
        "Creating Redis pool with reconnection policy"
    );

    let redis_pool = Pool::new(
        config,
        None,
        Some(connection_config),
        Some(re_connection_policy),
        6,
    )
    .map_err(|e| {
        error!(error = ?e, "Failed to create Redis pool");
        e
    })?;

    // Connects the connection pool to the Redis server.
    let redis_connection = redis_pool.connect();

    // Await that the whole pool is connected.
    redis_pool.wait_for_connect().await.map_err(|e| {
        error!(error = ?e, "Failed to connect to Redis server");
        e
    })?;

    info!("Redis connection pool successfully established");

    Ok((redis_pool, redis_connection))
}
