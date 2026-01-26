use axum::{http::StatusCode, middleware, routing::get, Router};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::middlewares::{http_metrics, request_id_middleware};
use crate::modules::{auth_v1, category_v1, feed_v1, media_v1, post_v1, tag_v1};

#[cfg(feature = "auth-oauth")]
use crate::modules::google_auth_v1;

#[cfg(feature = "user-management")]
use crate::modules::{email_verification_v1, forgot_password_v1, user_v1};

#[cfg(feature = "comments")]
use crate::modules::post_comment_v1;

#[cfg(feature = "newsletter")]
use crate::modules::newsletter_v1;

#[cfg(feature = "analytics")]
use crate::modules::analytics_v1;

#[cfg(feature = "admin-acl")]
use crate::modules::admin_acl_v1;

#[cfg(feature = "admin-routes")]
use crate::modules::admin_route_v1;

#[cfg(feature = "seed-system")]
use crate::modules::seed_v1;

use super::AppState;

pub fn router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/healthz", get(health_check))
        .nest("/auth/v1", auth_v1::routes());

    #[cfg(feature = "auth-oauth")]
    {
        router = router.nest("/auth/google/v1", google_auth_v1::routes());
    }

    #[cfg(feature = "user-management")]
    {
        router = router
            .nest("/user/v1", user_v1::routes())
            .nest("/email_verification/v1", email_verification_v1::routes())
            .nest("/forgot_password/v1", forgot_password_v1::routes());
    }

    router = router
        .nest("/post/v1", post_v1::routes());

    #[cfg(feature = "comments")]
    {
        router = router.nest("/post/comment/v1", post_comment_v1::routes());
    }

    router = router
        .nest("/category/v1", category_v1::routes())
        .nest("/tag/v1", tag_v1::routes())
        .nest("/media/v1", media_v1::routes())
        .nest("/feed/v1", feed_v1::routes());

    #[cfg(feature = "newsletter")]
    {
        router = router.nest("/newsletter/v1", newsletter_v1::routes());
    }

    #[cfg(feature = "analytics")]
    {
        router = router.nest("/analytics/v1", analytics_v1::routes());
    }

    #[cfg(feature = "admin-routes")]
    {
        router = router.nest("/admin/route/v1", admin_route_v1::routes());
    }

    #[cfg(feature = "admin-acl")]
    {
        router = router.nest("/admin/acl/v1", admin_acl_v1::routes());
    }

    #[cfg(feature = "seed-system")]
    {
        router = router.nest("/admin/seed/v1", seed_v1::routes());
    }

    router
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
