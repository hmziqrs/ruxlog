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
    let admin = Router::new()
        .route("/create", post(controller::create))
        .route("/update/{category_id}", post(controller::update))
        .route("/delete/{category_id}", post(controller::delete))
        .route("/list/query", post(controller::find_with_query))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    let public = Router::new()
        .route("/list", get(controller::find_all))
        .route("/view/{category_id}", get(controller::find_by_id_or_slug));

    admin.merge(public)
}
