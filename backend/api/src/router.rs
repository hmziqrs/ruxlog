use axum::{
    extract::State,
    http::{header, StatusCode},
    middleware,
    routing::{get, post},
    Json, Router,
};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::middlewares::{http_metrics, rate_limit, request_id_middleware, security_headers};
use crate::modules::{
    auth_v1, category_v1, csrf_v1, feed_v1, media_v1, post_v1, search_v1, tag_v1, user_v1,
};
use fred::interfaces::ClientLike;

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

use crate::utils::sanitize::xml_escape;

use super::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let mut router = Router::new()
        .route("/healthz", get(health_check))
        .route("/robots.txt", get(robots_txt))
        .route("/sitemap.xml", get(sitemap_xml))
        // Per-session CSRF token issuer. Lives inside the main router so the
        // SessionManagerLayer and csrf_guard both apply (it's exempted from the
        // latter via the exact-match exempt list). The handler bootstraps a new
        // session and returns a token bound to its id.
        .route("/csrf/v1/generate", post(csrf_v1::controller::generate))
        .nest(
            "/auth/v1",
            auth_v1::routes().layer(rate_limit::RateLimitLayer::new(state.clone(), 100, 60)),
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

    // DOS-TRACKVIEW-1: the public `track_view` endpoint writes a DB transaction
    // per call, so the whole post nest gets a generous per-IP cap (reads are
    // cheap SELECTs; 200/min/IP is ample for a real user and bounds a flooder
    // that previously hit the txn-per-request view counter unbounded).
    router = router.nest(
        "/post/v1",
        post_v1::routes().layer(rate_limit::RateLimitLayer::new(state.clone(), 200, 60)),
    );

    #[cfg(feature = "comments")]
    {
        router = router.nest(
            "/post/comment/v1",
            post_comment_v1::routes().layer(rate_limit::RateLimitLayer::new(
                state.clone(),
                100,
                60,
            )), // 100 req/min
        );
    }

    router = router
        .nest("/category/v1", category_v1::routes())
        .nest("/tag/v1", tag_v1::routes())
        // DOS-MEDIA-OPTIMIZER: the upload path runs the CPU-heavy image
        // optimizer; give /media/v1 its own tight per-IP cap (it is also
        // author-gated) so a burst of uploads cannot monopolize workers.
        .nest(
            "/media/v1",
            media_v1::routes().layer(rate_limit::RateLimitLayer::new(state.clone(), 30, 60)),
        )
        .nest("/feed/v1", feed_v1::routes())
        // DOS-SEARCH-1: search runs a triple leading-wildcard ILIKE (full table
        // scan) per request and was previously un-rate-limited. 30/min/IP bounds
        // an anonymous caller cheaply minting a CSRF token then replaying it.
        .nest(
            "/search/v1",
            search_v1::routes().layer(rate_limit::RateLimitLayer::new(state.clone(), 30, 60)),
        );

    #[cfg(feature = "newsletter")]
    {
        router = router.nest(
            "/newsletter/v1",
            newsletter_v1::routes().layer(rate_limit::RateLimitLayer::new(state.clone(), 100, 60)), // 100 req/min
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

    #[cfg(feature = "openapi")]
    {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;

        router = router.merge(
            SwaggerUi::new("/api/docs").url("/api/docs.json", crate::docs::ApiDoc::openapi()),
        );
    }

    router
        .layer(middleware::from_fn(security_headers::security_headers))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(http_metrics::track_metrics))
        .layer(
            // Do NOT capture headers in spans/responses: Cookie, Authorization,
            // csrf-token, and webhook-signature would otherwise ship to OTLP at
            // INFO. `include_headers(false)` is the tower-http default but is set
            // explicitly here so the safety property is self-documenting and not
            // contingent on knowing the library default. See plan Phase 2b / V-MED-7.
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(false),
                )
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
}

async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let db_status = match state.sea_db.ping().await {
        Ok(()) => "ok",
        Err(_) => "error",
    };

    let redis_status: &str = match state.redis_pool.ping::<()>(None).await {
        Ok(_) => "ok",
        Err(_) => "error",
    };

    let healthy = db_status == "ok" && redis_status == "ok";
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

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

async fn robots_txt() -> (
    StatusCode,
    [(axum::http::HeaderName, &'static str); 1],
    &'static str,
) {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        "User-agent: *\nAllow: /\nDisallow: /admin/\nDisallow: /api/\nDisallow: /auth/\n\nSitemap: https://ruxlog.com/sitemap.xml\n",
    )
}

async fn sitemap_xml(
    State(state): State<AppState>,
) -> Result<
    (
        StatusCode,
        [(axum::http::HeaderName, &'static str); 1],
        String,
    ),
    StatusCode,
> {
    use crate::db::sea_models::post;

    let posts = post::Entity::sitemap(&state.sea_db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let raw_base_url =
        std::env::var("CONSUMER_SITE_URL").unwrap_or_else(|_| "https://ruxlog.com".to_string());
    // SITEMAP-XML-1: escape both the operator base URL and every post slug
    // before interpolating into XML. Slugs are author-controlled and only
    // length-validated, so unescaped interpolation is a stored XML-injection
    // vector. `feed_v1` already escapes its RSS output the same way.
    let base_url = xml_escape(&raw_base_url);

    let mut urls = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    urls.push_str(&format!(
        "  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
        base_url
    ));

    for p in &posts {
        let lastmod = p.updated_at.to_rfc3339();
        let slug = xml_escape(&p.slug);
        urls.push_str(&format!(
            "  <url><loc>{}/posts/{}</loc><lastmod>{}</lastmod><changefreq>weekly</changefreq><priority>0.8</priority></url>\n",
            base_url, slug, lastmod
        ));
    }

    urls.push_str("</urlset>");

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        urls,
    ))
}
