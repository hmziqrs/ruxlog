pub mod controller;
pub mod validator;

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/upload", post(controller::upload))
        .route("/:asset_id", put(controller::update))
        .route("/:asset_id", delete(controller::delete))
        .route("/:asset_id", get(controller::find_by_id))
        // .route("/", get(controller::find_all))
        .route("/query", get(controller::find_with_query))
}