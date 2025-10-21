use axum::{http::StatusCode, middleware, routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::modules::post_comment_v1;
use crate::{
    middlewares::route_blocker::block_routes,
    modules::{asset_v1, category_v1, feed_v1, media_v1, newsletter_v1, post_v1, seed_v1, tag_v1},
};

use super::{
    modules::{auth_v1, email_verification_v1, forgot_password_v1, user_v1},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(health_check))
        .layer(middleware::from_fn(block_routes))
        .nest("/auth/v1", auth_v1::routes())
        .nest("/user/v1", user_v1::routes())
        .nest("/email_verification/v1", email_verification_v1::routes())
        .nest("/forgot_password/v1", forgot_password_v1::routes())
        .nest("/post/v1", post_v1::routes())
        .nest("/post/comment/v1", post_comment_v1::routes())
        .nest("/category/v1", category_v1::routes())
        .nest("/tag/v1", tag_v1::routes())
        .nest("/asset/v1", asset_v1::routes())
        .nest("/media/v1", media_v1::routes())
        .nest("/feed/v1", feed_v1::routes())
        .nest("/newsletter/v1", newsletter_v1::routes())
        .nest("/admin/seed/v1", seed_v1::routes())
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> StatusCode {
    StatusCode::NO_CONTENT
}
