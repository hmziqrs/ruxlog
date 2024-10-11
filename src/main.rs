pub mod db;
pub mod modules;
mod router;
pub mod state;

use std::env;

pub use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting server.");
    let pool = db::connect::get_pool();
    tracing::info!("Postgres connection established.");
    let state = AppState { db_pool: pool };

    // build our application with a single route
    let app = router::router().with_state(state);

    // run our app with hyper, listening globally on port 3000
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let address = format!("{}:{}", host, port);
    tracing::info!("Listening on http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
