pub mod controller;
pub mod validator;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    let admin = Router::<AppState>::new()
        .route("/block", post(controller::block_route))
        .route("/unblock", post(controller::unblock_route))
        .route("/update", post(controller::update_route_status))
        .route("/delete", post(controller::delete_route))
        .route("/list", post(controller::list_routes))
        .route("/sync", get(controller::sync_routes_to_redis))
        .route(
            "/sync_interval",
            get(controller::get_sync_interval).post(controller::update_sync_interval),
        )
        .route(
            "/sync_interval/pause",
            post(controller::pause_sync_interval),
        )
        .route(
            "/sync_interval/resume",
            post(controller::resume_sync_interval),
        )
        .route(
            "/sync_interval/restart",
            post(controller::restart_sync_interval),
        )
        .route_layer(middleware::from_fn(auth_guard::verified_with_role::<{ auth_guard::ROLE_ADMIN }>));

    admin
}
