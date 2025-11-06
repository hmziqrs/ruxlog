pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

/// Routes for the analytics v1 module.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/user/registration-trends",
            post(controller::registration_trends),
        )
        .route(
            "/user/verification-rates",
            post(controller::verification_rates),
        )
        .route(
            "/content/publishing-trends",
            post(controller::publishing_trends),
        )
        .route("/engagement/page-views", post(controller::page_views))
        .route("/engagement/comment-rate", post(controller::comment_rate))
        .route(
            "/engagement/newsletter-growth",
            post(controller::newsletter_growth),
        )
        .route(
            "/media/upload-trends",
            post(controller::media_upload_trends),
        )
        .route("/dashboard/summary", post(controller::dashboard_summary))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
}
