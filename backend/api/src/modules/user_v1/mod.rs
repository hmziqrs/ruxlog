pub mod controller;
pub mod validator;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    // Only verified users can update
    let base = Router::<AppState>::new()
        .route("/update", post(controller::update_profile))
        .route_layer(middleware::from_fn(auth_guard::verified))
        // Any authenticated user can get their profile
        .merge(
            Router::<AppState>::new()
                .route("/get", get(controller::get_profile))
                .route_layer(middleware::from_fn(auth_guard::authenticated)),
        );

    let admin = Router::<AppState>::new()
        .route("/list", post(controller::admin_list))
        .route("/view/{user_id}", post(controller::admin_view))
        .route("/create", post(controller::admin_create))
        .route("/update/{user_id}", post(controller::admin_update))
        .route("/delete/{user_id}", post(controller::admin_delete))
        .route_layer(middleware::from_fn(auth_guard::verified_with_role::<{ auth_guard::ROLE_ADMIN }>));

    base.nest("/admin", admin)
}
