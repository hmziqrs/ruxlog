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
    // Only verified users can update
    let base = Router::new()
        .route("/update", post(controller::update_profile))
        .route_layer(middleware::from_fn(user_status::only_verified))
        // Any authenticated user can get their profile
        .merge(Router::new().route("/get", get(controller::get_profile)))
        .route_layer(login_required!(AuthBackend));

    let admin = Router::new()
        .route("/list", post(controller::admin_list))
        .route("/view/{user_id}", post(controller::admin_view))
        .route("/create", post(controller::admin_create))
        .route("/update/{user_id}", post(controller::admin_update))
        .route("/delete/{user_id}", post(controller::admin_delete))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    base.nest("/admin", admin)
}
