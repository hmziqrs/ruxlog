pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};

use crate::{middlewares::user_status, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/request", post(controller::generate))
        .route("/verify", post(controller::verify))
        .route("/reset", post(controller::reset))
        .route_layer(middleware::from_fn(user_status::only_unauthenticated))
}
