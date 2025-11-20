use std::time::Duration;

use tower_sessions_redis_store::fred::prelude::*;
use tracing::{error, info, instrument};

use crate::core::config::CoreConfig;

#[instrument(skip(config))]
pub fn redis_config_from_core(config: &CoreConfig) -> Config {
    info!(
        redis_host = %config.redis_host,
        redis_port = config.redis_port,
        "Configuring Redis connection (core)"
    );

    Config {
        username: Some(config.redis_username.clone()),
        password: Some(config.redis_password.clone()),
        server: ServerConfig::Centralized {
            server: Server::new(config.redis_host.clone(), config.redis_port),
        },
        ..Default::default()
    }
}

#[instrument(skip(config))]
pub async fn init_redis_pool(config: &CoreConfig) -> Pool {
    let redis_config = redis_config_from_core(config);
    let connection_config = ConnectionConfig {
        reconnect_on_auth_error: true,
        connection_timeout: Duration::from_millis(1500),
        ..ConnectionConfig::default()
    };

    let reconnect_policy = ReconnectPolicy::new_linear(30, 1000 * 600, 500);

    info!(
        pool_size = 6,
        reconnect_attempts = 30,
        max_reconnect_delay_ms = 600000,
        "Creating Redis pool with reconnection policy (core)"
    );

    let redis_pool = Pool::new(
        redis_config,
        None,
        Some(connection_config),
        Some(reconnect_policy),
        6,
    )
    .map_err(|e| {
        error!(error = ?e, "Failed to create Redis pool (core)");
        e
    })
    .expect("Failed to create Redis pool (core)");

    redis_pool
}

