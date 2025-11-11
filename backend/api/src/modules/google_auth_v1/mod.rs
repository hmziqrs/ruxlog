pub mod controller;
pub mod service;
pub mod validator;

use axum::{routing::get, Router};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(controller::google_login))
        .route("/callback", get(controller::google_callback))
        .route("/user", get(controller::google_user_info))
}
