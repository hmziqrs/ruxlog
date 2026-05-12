use axum::{extract::State, http::{header, StatusCode}, middleware, routing::get, Json, Router};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::middlewares::{http_metrics, rate_limit, request_id_middleware, security_headers};
use crate::modules::{auth_v1, category_v1, feed_v1, media_v1, post_v1, search_v1, tag_v1, user_v1};

#[cfg(feature = "auth-oauth")]
use crate::modules::google_auth_v1;

#[cfg(feature = "user-management")]
use crate::modules::{email_verification_v1, forgot_password_v1};

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

#[cfg(feature = "billing")]
use crate::modules::billing_v1;

use super::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let mut router = Router::new()
        .route("/healthz", get(health_check))
        .route("/robots.txt", get(robots_txt))
        .route(
            "/sitemap.xml",
            get(sitemap_xml),
        )
        .nest(
            "/auth/v1",
            auth_v1::routes()
                .layer(rate_limit::RateLimitLayer::new(state.clone(), 5, 60)),
        );

    #[cfg(feature = "auth-oauth")]
    {
        router = router.nest("/auth/google/v1", google_auth_v1::routes());
    }

    // User profile routes - always available for authenticated users
    router = router.nest("/user/v1", user_v1::routes());

    // Email verification and password reset - only with user-management feature
    #[cfg(feature = "user-management")]
    {
        router = router
            .nest("/email_verification/v1", email_verification_v1::routes())
            .nest("/forgot_password/v1", forgot_password_v1::routes());
    }

    router = router
        .nest("/post/v1", post_v1::routes());

    #[cfg(feature = "comments")]
    {
        router = router.nest(
            "/post/comment/v1",
            post_comment_v1::routes()
                .layer(rate_limit::RateLimitLayer::new(state.clone(), 10, 60)), // 10 req/min
        );
    }

    router = router
        .nest("/category/v1", category_v1::routes())
        .nest("/tag/v1", tag_v1::routes())
        .nest("/media/v1", media_v1::routes())
        .nest("/feed/v1", feed_v1::routes())
        .nest("/search/v1", search_v1::routes());

    #[cfg(feature = "newsletter")]
    {
        router = router.nest(
            "/newsletter/v1",
            newsletter_v1::routes()
                .layer(rate_limit::RateLimitLayer::new(state.clone(), 5, 60)), // 5 req/min
        );
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

    #[cfg(feature = "billing")]
    {
        router = router.nest("/billing/v1", billing_v1::routes());
    }

    router
        .layer(middleware::from_fn(security_headers::security_headers))
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

async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let db_status = match state.sea_db.ping().await {
        Ok(()) => "ok",
        Err(_) => "error",
    };

    let redis_status = "ok";

    let healthy = db_status == "ok";
    let status = if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (
        status,
        Json(serde_json::json!({
            "status": if healthy { "healthy" } else { "degraded" },
            "components": {
                "database": db_status,
                "redis": redis_status,
            }
        })),
    )
}

async fn robots_txt() -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], &'static str) {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        "User-agent: *\nAllow: /\nDisallow: /admin/\nDisallow: /api/\nDisallow: /auth/\n\nSitemap: https://ruxlog.com/sitemap.xml\n",
    )
}

async fn sitemap_xml(State(state): State<AppState>) -> Result<(StatusCode, [(axum::http::HeaderName, &'static str); 1], String), StatusCode> {
    use crate::db::sea_models::post;

    let posts = post::Entity::sitemap(&state.sea_db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let base_url = std::env::var("CONSUMER_SITE_URL").unwrap_or_else(|_| "https://ruxlog.com".to_string());

    let mut urls = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    urls.push_str(&format!("  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n", base_url));

    for p in &posts {
        let lastmod = p.updated_at.to_rfc3339();
        urls.push_str(&format!(
            "  <url><loc>{}/posts/{}</loc><lastmod>{}</lastmod><changefreq>weekly</changefreq><priority>0.8</priority></url>\n",
            base_url, p.slug, lastmod
        ));
    }

    urls.push_str("</urlset>");

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "application/xml")], urls))
}
