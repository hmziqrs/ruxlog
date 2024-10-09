use axum::{
    response::Json, routing, Router
};
use serde_json::{Value, json};


use crate::modules;

pub fn router() -> Router {
    let auth_v1_routes: Router = Router::new()
        .route("/log_in", routing::post(modules::auth_v1::controller::login));

    return Router::new()
        .route("/", routing::get(handler))
        .nest("/auth/v1", auth_v1_routes);
        
}


async fn handler() -> Json<Value> {
    Json(json!({
        "message": "hot reload testing-xxxsadasdas!"
    }))
}
