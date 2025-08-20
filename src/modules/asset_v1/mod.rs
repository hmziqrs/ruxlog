pub mod controller;
pub mod validator;

use axum::{
    middleware,
    routing::{get, post},
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
        .route("/upload", post(controller::upload))
        .route("/update/{asset_id}", post(controller::update))
        .route("/delete/{asset_id}", post(controller::delete))
        .route("/{asset_id}", get(controller::find_by_id))
        .route("/query", post(controller::find_with_query))
        .route("/contexts", get(controller::contexts))
        .route_layer(middleware::from_fn(user_permission::author))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
