use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::{error, info, instrument};

use crate::core::config::CoreConfig;

#[instrument(skip(config))]
pub async fn init_db(config: &CoreConfig) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(config.postgres_url.clone());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        // Disable SQLx logging for the core/TUI path so it
        // does not interfere with the terminal UI.
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info);

    let conn = match Database::connect(opt).await {
        Ok(conn) => {
            info!("SeaORM database connection established (core)");
            conn
        }
        Err(e) => {
            error!("Failed to connect to database with SeaORM (core): {:?}", e);
            panic!("Failed to connect to database with SeaORM (core): {:?}", e);
        }
    };

    if let Err(e) = conn.ping().await {
        error!("Failed to ping database with SeaORM (core): {:?}", e);
        panic!("Failed to ping database with SeaORM (core): {:?}", e);
    }

    info!("SeaORM database connection working (core)");

    conn
}
