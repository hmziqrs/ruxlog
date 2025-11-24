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
    Router::new()
        .route("/list", get(controller::list_constants))
        .route("/get/{key}", get(controller::get_constant))
        .route("/create", post(controller::create_constant))
        .route("/update/{key}", post(controller::update_constant))
        .route("/delete/{key}", delete(controller::delete_constant))
        .route("/sync", post(controller::sync_constants))
        .route("/import_env", post(controller::import_env_constants))
        .route_layer(middleware::from_fn(user_permission::super_admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
