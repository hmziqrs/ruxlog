pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};
use axum_login::login_required;

use crate::{middlewares::user_status, services::auth::AuthBackend, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/verify", post(controller::verify))
        .route("/resend", post(controller::resend))
        .route_layer(middleware::from_fn(user_status::only_unverified))
        .route_layer(login_required!(AuthBackend))
}
