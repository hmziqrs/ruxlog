use axum::{http::StatusCode, middleware, routing::get, Router};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::middlewares::{http_metrics, request_id_middleware};
use crate::modules::post_comment_v1;
use crate::{
    middlewares::route_blocker::block_routes,
    modules::{
        admin_acl_v1, admin_route_v1, analytics_v1, category_v1, feed_v1, media_v1, newsletter_v1,
        post_v1, seed_v1, tag_v1,
    },
};

use super::{
    modules::{auth_v1, email_verification_v1, forgot_password_v1, google_auth_v1, user_v1},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(health_check))
        .layer(middleware::from_fn(block_routes))
        .nest("/auth/v1", auth_v1::routes())
        .nest("/auth/google/v1", google_auth_v1::routes())
        .nest("/user/v1", user_v1::routes())
        .nest("/email_verification/v1", email_verification_v1::routes())
        .nest("/forgot_password/v1", forgot_password_v1::routes())
        .nest("/post/v1", post_v1::routes())
        .nest("/post/comment/v1", post_comment_v1::routes())
        .nest("/category/v1", category_v1::routes())
        .nest("/tag/v1", tag_v1::routes())
        .nest("/media/v1", media_v1::routes())
        .nest("/feed/v1", feed_v1::routes())
        .nest("/newsletter/v1", newsletter_v1::routes())
        .nest("/analytics/v1", analytics_v1::routes())
        .nest("/admin/route/v1", admin_route_v1::routes())
        .nest("/admin/acl/v1", admin_acl_v1::routes())
        .nest("/admin/seed/v1", seed_v1::routes())
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(http_metrics::track_metrics))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true),
                )
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis)
                        .include_headers(true),
                ),
        )
}

async fn health_check() -> StatusCode {
    StatusCode::NO_CONTENT
}
