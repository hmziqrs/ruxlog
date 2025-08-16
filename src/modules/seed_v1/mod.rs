pub mod controller;

use axum::{routing::post, Router};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/seed_tags", post(controller::seed_tags))
        .route("/seed_categories", post(controller::seed_categories))
        .route("/seed_posts", post(controller::seed_posts))
        .route("/seed_post_comments", post(controller::seed_post_comments))
        .route("/seed", post(controller::seed))
}
