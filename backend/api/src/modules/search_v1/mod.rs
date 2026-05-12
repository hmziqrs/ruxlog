pub mod controller;
pub mod validator;

use axum::{routing::post, Router};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/search", post(controller::search))
}
