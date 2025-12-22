pub mod controller;
pub mod validator;

use axum::{extract::DefaultBodyLimit, middleware, routing::post, Router};

use crate::{config, middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    let media_limited = Router::<AppState>::new()
        .route("/create", post(controller::create))
        .layer(DefaultBodyLimit::max(config::body_limits::MEDIA));

    Router::<AppState>::new()
        .route("/view/{media_id}", post(controller::view))
        .route("/list/query", post(controller::find_with_query))
        .route("/usage/details", post(controller::list_usage_details))
        .route("/delete/{media_id}", post(controller::delete))
        .merge(media_limited)
        .route_layer(middleware::from_fn(auth_guard::verified_with_role::<{ auth_guard::ROLE_AUTHOR }>))
}
