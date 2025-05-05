pub mod controller;
pub mod validator;

use axum::{
    middleware, routing::{delete, get, post, put}, Router
};
use axum_login::login_required;

use crate::{middlewares::{user_permission, user_status}, services::auth::AuthBackend, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/upload", post(controller::upload))
        .route("/{asset_id}", put(controller::update))
        .route("/{asset_id}", delete(controller::delete))
        .route("/{asset_id}", get(controller::find_by_id))
        // .route("/", get(controller::find_all))
        .route("/query", get(controller::find_with_query))
        .route_layer(middleware::from_fn(user_permission::author))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
