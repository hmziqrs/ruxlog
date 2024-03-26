use axum::{
    routing::get,
    response::Json,
    Router,
};
use serde_json::{Value, json};

mod modules;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app: Router = Router::new().route("/", get(handler))
        .merge(modules::routes());
    

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handler() -> Json<Value> {
    Json(json!({
        "message": "hot reload testing-xxxsadasdas!"
    }))
}
