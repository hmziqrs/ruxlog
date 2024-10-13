use std::env;

use tokio::task::JoinHandle;
use tower_sessions_redis_store::fred::{
    error::RedisError,
    prelude::{ClientLike, RedisPool},
    types::{RedisConfig, Server, ServerConfig},
};

// Minimal Redis configuration.
fn redis_config() -> RedisConfig {
    RedisConfig {
        username: Some(env::var("REDIS_USERNAME").expect("REDIS_USERNAME must be set")),
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

    let redis_pool = RedisPool::new(config, None, None, None, 6)?;

    // Connects the connection pool to the Redis server.
    let redis_connection = redis_pool.connect();

    // Await that the whole pool is connected.
    redis_pool.wait_for_connect().await?;

    Ok((redis_pool, redis_connection))
}
