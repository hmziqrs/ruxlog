pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    let public = Router::<AppState>::new()
        .route("/register", post(controller::register))
        .route("/log_in", post(controller::log_in))
        .route_layer(middleware::from_fn(auth_guard::unauthenticated));

    let authenticated = Router::<AppState>::new()
        .route("/log_out", post(controller::log_out))
        .route("/2fa/setup", post(controller::twofa_setup))
        .route("/2fa/verify", post(controller::twofa_verify))
        .route("/2fa/disable", post(controller::twofa_disable))
        .route("/sessions/list", post(controller::sessions_list))
        .route(
            "/sessions/terminate/{id}",
            post(controller::sessions_terminate),
        )
        .route_layer(middleware::from_fn(auth_guard::authenticated));

    public.merge(authenticated)
}
