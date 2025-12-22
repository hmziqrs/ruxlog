pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/request", post(controller::generate))
        .route("/verify", post(controller::verify))
        .route("/reset", post(controller::reset))
        .route_layer(middleware::from_fn(auth_guard::unauthenticated))
}
