pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(controller::create))
        .route("/list/query", post(controller::find_with_query))
        .route("/delete/{media_id}", post(controller::delete))
        .route_layer(middleware::from_fn(user_permission::author))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
