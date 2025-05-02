use clap::{CommandFactory, Parser, Subcommand};
use migration::{Migrator, MigratorTrait};
use sea_orm_migration::prelude::*;
use std::env;

#[derive(Parser)]
#[command(name = "migrate")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Up,
    Down {
        #[arg(short, long)]
        steps: Option<u32>,
    },
    Fresh,
    Refresh,
    Status,
}

async fn get_db_url() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");

    format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db)
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let db_url = get_db_url().await;
    let connection = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    match cli.command {
        Some(Commands::Up) => {
            println!("Running migrations up...");
            Migrator::up(&connection, None)
                .await
                .expect("Failed to run migrations");
            println!("Migrations completed successfully!");
        }
        Some(Commands::Down { steps }) => {
            println!("Running migrations down...");
            Migrator::down(&connection, steps)
                .await
                .expect("Failed to revert migrations");
            println!("Migration reversion completed successfully!");
        }
        Some(Commands::Fresh) => {
            println!("Running fresh migrations...");
            Migrator::fresh(&connection)
                .await
                .expect("Failed to run fresh migrations");
            println!("Fresh migrations completed successfully!");
        }
        Some(Commands::Refresh) => {
            println!("Refreshing migrations...");
            Migrator::refresh(&connection)
                .await
                .expect("Failed to refresh migrations");
            println!("Migration refresh completed successfully!");
        }
        Some(Commands::Status) => {
            println!("Checking migration status...");
            let applied_migrations = Migrator::get_applied_migrations(&connection)
                .await
                .expect("Failed to get applied migrations");
            
            println!("Applied migrations:");
            for migration in applied_migrations {
                println!("  {}", migration);
            }
            
            println!("Pending migrations:");
            let pending_migrations = Migrator::get_pending_migrations(&connection)
                .await
                .expect("Failed to get pending migrations");
            
            for migration in pending_migrations {
                println!("  {}", migration);
            }
        }
        None => {
            let mut app = Cli::command();
            app.print_help().expect("Failed to print help");
        }
    }
}
