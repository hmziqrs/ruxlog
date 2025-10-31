pub mod controller;
pub mod service;
pub mod validator;

use axum::{middleware, routing::post, Router};

use crate::{middlewares::user_permission, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", post(controller::health_check))
        .route("/logs/search", post(controller::search_logs))
        .route("/logs/recent", post(controller::recent_logs))
        .route("/metrics/summary", post(controller::metrics_summary))
        .route("/stats/errors", post(controller::error_stats))
        .route("/stats/latency", post(controller::latency_stats))
        .route("/stats/auth", post(controller::auth_stats))
        .route_layer(middleware::from_fn(user_permission::admin))
}
