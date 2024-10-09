use axum::{
    response::Json, routing, Router
};
use serde_json::{Value, json};


use crate::modules;

pub fn router() -> Router {
    Router::new().route("/", routing::get(handler))
        .route("/auth/v1/log_in", routing::post(modules::auth_v1::controller::login))
}



async fn handler() -> Json<Value> {
    Json(json!({
        "message": "hot reload testing-xxxsadasdas!"
    }))
}
