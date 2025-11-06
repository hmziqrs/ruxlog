pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

/// Routes for the analytics v1 module.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/user/registration-trends",
            post(controller::registration_trends),
        )
        .route(
            "/user/verification-rates",
            post(controller::verification_rates),
        )
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
