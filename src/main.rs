mod router;
pub mod modules;
pub mod db;
pub mod state;

pub use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing::info!("Starting server.");
    let pool = db::connect::get_pool();
    tracing::info!("Postgres connection established.");
    let state = AppState {
        db_pool: pool,
    };

    // build our application with a single route
    let app = router::router().with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

