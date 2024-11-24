use std::{env, time::Duration};

use tokio::task::JoinHandle;
use tower_sessions_redis_store::fred::{
    error::RedisError,
    prelude::{ClientLike, RedisPool},
    types::{ConnectionConfig, ReconnectPolicy, RedisConfig, Server, ServerConfig},
};

// Minimal Redis configuration.
fn redis_config() -> RedisConfig {
    RedisConfig {
        username: Some(env::var("REDIS_USER").expect("REDIS_USER must be set")),
        password: Some(env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set")),
        server: ServerConfig::Centralized {
            server: Server::new(
                env::var("REDIS_HOST").expect("REDIS_HOST must be set"),
                env::var("REDIS_PORT")
                    .expect("REDIS_PORT must be set")
                    .parse()
                    .expect("REDIS_PORT must be a valid u16"),
            ),
        },
        ..Default::default()
    }
}

/// Setup the Redis connection pool.
pub async fn init_redis_store(
) -> Result<(RedisPool, JoinHandle<Result<(), RedisError>>), RedisError> {
    let config = redis_config();
    let connection_config = ConnectionConfig {
        reconnect_on_auth_error: true,
        connection_timeout: Duration::from_millis(1500),

        ..ConnectionConfig::default()
    };

    let re_connection_policy = ReconnectPolicy::new_linear(30, 1000 * 600, 500);

    let redis_pool = RedisPool::new(
        config,
        None,
        Some(connection_config),
        // None,
        Some(re_connection_policy),
        6,
    )?;

    // Connects the connection pool to the Redis server.
    let redis_connection = redis_pool.connect();

    // Await that the whole pool is connected.
    redis_pool.wait_for_connect().await?;

    Ok((redis_pool, redis_connection))
}
