use axum::{response::Json, routing, Router};
use serde_json::{json, Value};
use tower_http::trace::TraceLayer;
use tracing;

use crate::modules;

pub fn router() -> Router {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let auth_v1_routes: Router = Router::new().route(
        "/log_in",
        routing::post(modules::auth_v1::controller::login),
    );

    return Router::new()
        .route("/", routing::get(handler))
        .nest("/auth/v1", auth_v1_routes)
        .layer(TraceLayer::new_for_http());
}

async fn handler() -> Json<Value> {
    Json(json!({
        "message": "hot reload testing-xxxsadasdas!"
    }))
}
