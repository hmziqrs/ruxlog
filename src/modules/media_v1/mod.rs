pub mod controller;
pub mod validator;

use axum::{extract::DefaultBodyLimit, middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    config,
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

pub fn routes() -> Router<AppState> {
    let media_limited = Router::new()
        .route("/create", post(controller::create))
        .layer(DefaultBodyLimit::max(config::body_limits::MEDIA));

    Router::new()
        .route("/list/query", post(controller::find_with_query))
        .route("/delete/{media_id}", post(controller::delete))
        .merge(media_limited)
        .route_layer(middleware::from_fn(user_permission::author))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
