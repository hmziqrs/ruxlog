use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{env, time::Duration};
use tracing::{error, info, instrument};

/// Get the database URL from environment variables
#[instrument]
fn get_db_url() -> Result<String, String> {
    let user = env::var("POSTGRES_USER")
        .map_err(|_| "POSTGRES_USER environment variable must be set".to_string())?;
    let password = env::var("POSTGRES_PASSWORD")
        .map_err(|_| "POSTGRES_PASSWORD environment variable must be set".to_string())?;
    let db = env::var("POSTGRES_DB")
        .map_err(|_| "POSTGRES_DB environment variable must be set".to_string())?;
    let host = env::var("POSTGRES_HOST")
        .map_err(|_| "POSTGRES_HOST environment variable must be set".to_string())?;
    let port = env::var("POSTGRES_PORT")
        .map_err(|_| "POSTGRES_PORT environment variable must be set".to_string())?;

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db
    ))
}

fn connect_options(db_url: &str) -> ConnectOptions {
    let mut opt = ConnectOptions::new(db_url.to_string());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        // Disable SQLx logging to prevent noisy output in TUI;
        // Axum can enable tracing separately.
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info);
    opt
}

/// Establishes a connection to the database using SeaORM
/// This function panics on errors - for non-panicking version use `try_connect()`
#[instrument]
pub async fn init_db(run_migrations: bool) -> DatabaseConnection {
    match try_connect(run_migrations).await {
        Ok(conn) => {
            info!("SeaORM database connection working");
            conn
        }
        Err(e) => {
            error!("Database initialization failed: {}", e);
            panic!("Database initialization failed: {}", e);
        }
    }
}

/// Non-panicking helper to test DB connectivity (and optionally migrations).
pub async fn try_connect(run_migrations: bool) -> Result<DatabaseConnection, String> {
    let db_url = get_db_url()?;
    let opt = connect_options(&db_url);

    let conn = Database::connect(opt)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    conn.ping()
        .await
        .map_err(|e| format!("Failed to ping database: {}", e))?;

    if run_migrations {
        info!("Starting database migrations");
        Migrator::up(&conn, None)
            .await
            .map_err(|e| format!("Failed to run migrations: {}", e))?;
        info!("Database migrations completed successfully");
    }

    info!("SeaORM database connection established");
    Ok(conn)
}

/// Backwards-compatible helper for the Axum server.
#[instrument]
pub async fn get_sea_connection() -> DatabaseConnection {
    init_db(true).await
}
