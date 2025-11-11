pub mod controller;
pub mod validator;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use axum_login::login_required;

use crate::{
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

pub fn routes() -> Router<AppState> {
    let admin = Router::new()
        .route("/block", post(controller::block_route))
        .route("/unblock/{pattern}", post(controller::unblock_route))
        .route("/update/{pattern}", post(controller::update_route_status))
        .route("/delete/{pattern}", delete(controller::delete_route))
        .route("/list", get(controller::list_blocked_routes))
        .route("/sync", get(controller::sync_routes_to_redis))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    admin
}
