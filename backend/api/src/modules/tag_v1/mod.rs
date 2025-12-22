pub mod controller;
pub mod validator;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    let admin = Router::<AppState>::new()
        .route("/create", post(controller::create))
        .route("/update/{tag_id}", post(controller::update))
        .route("/delete/{tag_id}", post(controller::delete))
        .route("/view/{tag_id}", post(controller::find_by_id))
        .route("/list/query", post(controller::find_with_query))
        .route_layer(middleware::from_fn(auth_guard::verified_with_role::<{ auth_guard::ROLE_ADMIN }>));

    let public = Router::<AppState>::new().route("/list", get(controller::find_all));

    admin.merge(public)
}
