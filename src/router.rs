use axum::{
    extract::State,
    response::Json,
    routing::{self, post},
    Router,
};
use serde_json::{json, Value};
use tower_http::trace::TraceLayer;

use crate::{modules, AppState};

pub fn router() -> Router<AppState> {
    let auth_v1_routes = Router::new()
        .route("/register", post(modules::auth_v1::controller::register))
        .route("/log_in", post(modules::auth_v1::controller::log_in));

    return Router::new()
        .route("/", routing::get(handler))
        .nest("/auth/v1", auth_v1_routes)
        .layer(TraceLayer::new_for_http());
}

async fn handler(s: State<AppState>) -> Json<Value> {
    println!("{:?}", s.db_pool.status());
    Json(json!({
        "message": "hot reload testing-xxxsadasdas!"
    }))
}
