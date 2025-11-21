use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{env, time::Duration};
use tracing::{error, info, instrument};

/// Get the database URL from environment variables
#[instrument]
fn get_db_url() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");

    format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db)
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
#[instrument]
pub async fn init_db(run_migrations: bool) -> DatabaseConnection {
    let db_url = get_db_url();
    let opt = connect_options(&db_url);

    let conn = match Database::connect(opt).await {
        Ok(conn) => {
            info!("SeaORM database connection established");
            conn
        }
        Err(e) => {
            error!("Failed to connect to database with SeaORM: {:?}", e);
            panic!("Failed to connect to database with SeaORM: {:?}", e);
        }
    };

    if let Err(e) = conn.ping().await {
        error!("Failed to ping database with SeaORM: {:?}", e);
        panic!("Failed to ping database with SeaORM: {:?}", e);
    }

    if run_migrations {
        let migration_span = tracing::info_span!("database_migration");
        let _guard = migration_span.enter();

        info!("Starting database migrations");
        match Migrator::up(&conn, None).await {
            Ok(_) => {
                info!("Database migrations completed successfully");
                tracing::Span::current().record("result", "success");
            }
            Err(e) => {
                error!(error = ?e, "Failed to run database migrations");
                tracing::Span::current().record("result", "failed");
                panic!("Failed to run migrations: {:?}", e);
            }
        }
    }

    info!("SeaORM database connection working");

    conn
}

/// Backwards-compatible helper for the Axum server.
#[instrument]
pub async fn get_sea_connection() -> DatabaseConnection {
    init_db(true).await
}
