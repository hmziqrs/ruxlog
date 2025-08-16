pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};

use crate::{
    middlewares::user_status,
    AppState,
};

pub fn routes() -> Router<AppState> {
    let public = Router::new()
        .route("/register", post(controller::register))
        .route("/log_in", post(controller::log_in))
        .route_layer(middleware::from_fn(user_status::only_unauthenticated));

    let authenticated = Router::new()
        .route("/log_out", post(controller::log_out))
        .route("/2fa/setup", post(controller::twofa_setup))
        .route("/2fa/verify", post(controller::twofa_verify))
        .route("/2fa/disable", post(controller::twofa_disable))
        .route("/sessions/list", post(controller::sessions_list))
        .route(
            "/sessions/terminate/{id}",
            post(controller::sessions_terminate),
        )
        .route_layer(middleware::from_fn(user_status::only_authenticated));

    public.merge(authenticated)
}
