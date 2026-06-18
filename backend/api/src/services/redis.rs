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

    // Optional TLS (rediss) transport, gated by REDIS_TLS=true|1. Uses the
    // system trust store via rustls-native-certs. See plan Phase 6a.
    //
    // Hostname verification: fred verifies the server certificate's SAN against
    // the configured REDIS_HOST during the rustls handshake (the ServerName
    // passed to rustls is `server.tls_server_name.unwrap_or(server.host)`).
    // REDIS_TLS_SERVER_NAME may be set to override the expected hostname when
    // REDIS_HOST is an IP address or alias that does not match a SAN on the
    // certificate. The `TlsConfig.hostnames` field is deliberately left at its
    // default of `TlsHostMapping::None` here — per fred's API that field only
    // rewrites IP/hostname mappings in CLUSTER SLOTS responses for clustered
    // deployments and has no effect on the centralized connection's hostname
    // verification.
    let tls = if env::var("REDIS_TLS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
    {
        info!("Enabling TLS for the Redis connection (REDIS_TLS=true)");
        Some(
            TlsConnector::default_rustls()
                .expect(
                    "REDIS_TLS set but could not build a rustls connector from the native cert \
                     store",
                )
                .into(),
        )
    } else {
        None
    };

    // Optional override for the TLS ServerName when REDIS_HOST is not the name
    // on the certificate (e.g. an IP or internal alias).
    let tls_server_name = env::var("REDIS_TLS_SERVER_NAME").ok().filter(|s| !s.is_empty());

    Config {
        username: Some(env::var("REDIS_USER").expect("REDIS_USER must be set")),
        password: Some(env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set")),
        server: ServerConfig::Centralized {
            server: Server::new_with_tls(host, port, tls_server_name),
        },
        tls,
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

/// Initialize a Redis pool without wrapping it in a session store.
/// Useful for non-HTTP consumers like the TUI.
#[instrument(name = "redis_pool_init_simple")]
pub async fn init_redis_pool_only() -> Result<Pool, Error> {
    let (pool, _handle) = init_redis_store().await?;
    Ok(pool)
}
