pub mod controller;
pub mod validator;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/list", get(controller::list_constants))
        .route("/get/{key}", get(controller::get_constant))
        .route("/create", post(controller::create_constant))
        .route("/update/{key}", post(controller::update_constant))
        .route("/delete/{key}", delete(controller::delete_constant))
        .route("/sync", post(controller::sync_constants))
        .route("/import_env", post(controller::import_env_constants))
        .route_layer(middleware::from_fn(auth_guard::verified_with_role::<{ auth_guard::ROLE_SUPER_ADMIN }>))
}
