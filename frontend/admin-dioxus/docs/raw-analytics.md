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


use std::collections::{BTreeMap, HashMap};

use axum::{extract::State, response::IntoResponse, Json};
use axum_macros::debug_handler;
use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    sea_query::{ArrayType, Value},
    DatabaseBackend, FromQueryResult, Statement,
};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use tracing::instrument;

use crate::{
    db::sea_models::post::PostStatus,
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

use super::validator::{
    AnalyticsEnvelope, AnalyticsEnvelopeResponse, AnalyticsMeta, CommentRatePoint,
    CommentRateRequest, CommentRateSort, DashboardSummaryData, DashboardSummaryEngagement,
    DashboardSummaryMedia, DashboardSummaryPosts, DashboardSummaryRequest, DashboardSummaryUsers,
    MediaUploadPoint, MediaUploadRequest, NewsletterGrowthPoint, NewsletterGrowthRequest,
    PageViewPoint, PageViewsRequest, PublishingTrendPoint, PublishingTrendsRequest,
    RegistrationTrendPoint, RegistrationTrendsRequest, VerificationRatePoint,
    VerificationRatesRequest,
};

#[derive(Debug, FromQueryResult)]
struct RegistrationTrendRow {
    bucket: String,
    new_users: i64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct VerificationRateRow {
    bucket: String,
    requested: i64,
    verified: i64,
    success_rate: f64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct PublishingTrendRow {
    bucket: String,
    status: String,
    count: i64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct PageViewRow {
    bucket: String,
    views: i64,
    unique_visitors: i64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct CommentRateRow {
    post_id: i32,
    title: String,
    views: i64,
    comments: i64,
    comment_rate: f64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct NewsletterGrowthRow {
    bucket: String,
    new_subscribers: i64,
    confirmed: i64,
    unsubscribed: i64,
    net_growth: i64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct MediaUploadRow {
    bucket: String,
    upload_count: i64,
    total_size_mb: f64,
    avg_size_mb: f64,
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct DashboardSummaryRow {
    users_total: i64,
    users_new: i64,
    posts_published: i64,
    posts_drafts: i64,
    views_in_period: i64,
    comments_in_period: i64,
    newsletter_confirmed: i64,
    media_total: i64,
    media_uploads: i64,
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn registration_trends(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<RegistrationTrendsRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let interval = request.filters.group_by;
    let bucket_expr = interval.to_bucket_expr("users.created_at");
    let sort_field = match resolved.sort_by.as_deref() {
        Some("new_users") => "new_users",
        Some("bucket") => "bucket",
        Some("count") => "new_users",
        Some(_) => "bucket",
        None => "bucket",
    };

    let order_clause = if sort_field == "new_users" {
        format!(
            "ORDER BY new_users {} , bucket ASC",
            resolved.sort_order.as_sql()
        )
    } else {
        format!("ORDER BY bucket {}", resolved.sort_order.as_sql())
    };

    let sql = format!(
        r#"
        WITH bucketed AS (
            SELECT
                {bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS new_users
            FROM users
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY 1
        )
        SELECT bucket, new_users, COUNT(*) OVER () AS total
        FROM bucketed
        {order_clause}
        LIMIT $3 OFFSET $4
        "#,
        bucket_expr = bucket_expr,
        order_clause = order_clause,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
        ],
    );

    let rows = RegistrationTrendRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let data: Vec<RegistrationTrendPoint> = rows
        .into_iter()
        .map(|row| RegistrationTrendPoint {
            bucket: row.bucket,
            new_users: row.new_users,
        })
        .collect();

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_sorted_by(sort_field.to_string())
        .with_filters(json!({ "group_by": interval.as_str() }));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn verification_rates(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<VerificationRatesRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let interval = request.filters.group_by;
    let bucket_expr = interval.to_bucket_expr("email_verifications.created_at");
    let order_target = match resolved.sort_by.as_deref() {
        Some("requested") => "requested",
        Some("verified") => "verified",
        Some("success_rate") => "success_rate",
        Some("bucket") => "bucket",
        Some(_) => "bucket",
        None => "bucket",
    };

    let order_clause = if order_target == "bucket" {
        format!("ORDER BY bucket {}", resolved.sort_order.as_sql())
    } else {
        format!(
            "ORDER BY {order_target} {} , bucket ASC",
            resolved.sort_order.as_sql()
        )
    };

    let user_bucket_expr = interval.to_bucket_expr("users.updated_at");

    let sql = format!(
        r#"
        WITH requests AS (
            SELECT
                {bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS requested
            FROM email_verifications
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY 1
        ),
        verified AS (
            SELECT
                {user_bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS verified
            FROM users
            WHERE is_verified = TRUE
              AND updated_at >= $1
              AND updated_at <= $2
            GROUP BY 1
        ),
        combined AS (
            SELECT
                COALESCE(r.bucket, v.bucket) AS bucket,
                COALESCE(r.requested, 0) AS requested,
                COALESCE(v.verified, 0) AS verified
            FROM requests r
            FULL OUTER JOIN verified v ON r.bucket = v.bucket
        )
        SELECT
            bucket,
            requested,
            verified,
            CASE
                WHEN requested = 0 THEN 0::FLOAT8
                ELSE ROUND((verified::NUMERIC / requested::NUMERIC) * 100, 2)::FLOAT8
            END AS success_rate,
            COUNT(*) OVER () AS total
        FROM combined
        {order_clause}
        LIMIT $3 OFFSET $4
        "#,
        bucket_expr = bucket_expr,
        user_bucket_expr = user_bucket_expr,
        order_clause = order_clause,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
        ],
    );

    let rows = VerificationRateRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let data: Vec<VerificationRatePoint> = rows
        .into_iter()
        .map(|row| VerificationRatePoint {
            bucket: row.bucket,
            requested: row.requested,
            verified: row.verified,
            success_rate: row.success_rate,
        })
        .collect();

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_sorted_by(order_target.to_string())
        .with_filters(json!({ "group_by": interval.as_str() }));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn publishing_trends(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<PublishingTrendsRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let interval = request.filters.group_by;
    let bucket_expr = interval.to_bucket_expr("posts.created_at");
    let bucket_order = format!("ORDER BY bucket {}", resolved.sort_order.as_sql());

    let status_filter = parse_status_filters(request.filters.status.as_ref())?;
    let status_param = status_array_value(status_filter.as_ref());
    let requested_labels = status_filter.as_ref().map(|statuses| {
        statuses
            .iter()
            .map(status_label_from_enum)
            .collect::<Vec<_>>()
    });
    let has_status_filter = status_filter.is_some();

    let sql = format!(
        r#"
        WITH bucketed AS (
            SELECT
                {bucket_expr} AS bucket,
                status::text AS status,
                COUNT(*)::BIGINT AS count
            FROM posts
            WHERE created_at >= $1
              AND created_at <= $2
              AND ($5 IS NULL OR status::text = ANY($5))
            GROUP BY 1, status
        ),
        bucket_list AS (
            SELECT
                bucket,
                COUNT(*) OVER () AS total,
                ROW_NUMBER() OVER ({bucket_order}) AS rn
            FROM (
                SELECT DISTINCT bucket FROM bucketed
            ) distinct_bucket
        )
        SELECT
            b.bucket,
            b.total,
            bucketed.status,
            bucketed.count
        FROM bucket_list b
        JOIN bucketed ON bucketed.bucket = b.bucket
        WHERE b.rn > $4 AND b.rn <= $4 + $3
        {bucket_order}, bucketed.status ASC
        "#,
        bucket_expr = bucket_expr,
        bucket_order = bucket_order,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
            status_param,
        ],
    );

    let rows = PublishingTrendRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let mut ordered_buckets: Vec<String> = Vec::new();
    let mut bucket_counts: HashMap<String, BTreeMap<String, i64>> = HashMap::new();

    for row in &rows {
        let entry = bucket_counts.entry(row.bucket.clone()).or_insert_with(|| {
            ordered_buckets.push(row.bucket.clone());
            BTreeMap::new()
        });

        entry.insert(status_label_from_str(&row.status), row.count);
    }

    let mut data: Vec<PublishingTrendPoint> = Vec::new();

    for bucket in ordered_buckets {
        if let Some(counts) = bucket_counts.get(&bucket) {
            let mut filtered_counts = if let Some(labels) = &requested_labels {
                let mut map = BTreeMap::new();
                for label in labels {
                    let value = *counts.get(label).unwrap_or(&0);
                    map.insert(label.clone(), value);
                }
                map
            } else {
                counts.clone()
            };

            if let Some(labels) = &requested_labels {
                for label in labels {
                    filtered_counts.entry(label.clone()).or_insert(0);
                }
            }

            if has_status_filter && filtered_counts.values().all(|&value| value == 0) {
                continue;
            }

            data.push(PublishingTrendPoint {
                bucket: bucket.clone(),
                counts: filtered_counts,
            });
        }
    }

    let mut filters_obj = JsonMap::new();
    filters_obj.insert("group_by".into(), json!(interval.as_str()));
    if let Some(labels) = &requested_labels {
        filters_obj.insert("status".into(), json!(labels));
    }

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_sorted_by(
            resolved
                .sort_by
                .clone()
                .unwrap_or_else(|| "bucket".to_string()),
        )
        .with_filters(JsonValue::Object(filters_obj));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn page_views(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<PageViewsRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let filters = &request.filters;
    let interval = filters.group_by;
    let post_id_filter = filters.post_id;
    let author_id_filter = filters.author_id;
    let only_unique = filters.only_unique;
    let bucket_expr = interval.to_bucket_expr("pv.created_at");
    let order_clause = format!("ORDER BY bucket {}", resolved.sort_order.as_sql());

    let sql = format!(
        r#"
        WITH filtered AS (
            SELECT
                pv.post_id,
                pv.user_id,
                pv.ip_address,
                {bucket_expr} AS bucket
            FROM post_views pv
            LEFT JOIN posts p ON pv.post_id = p.id
            WHERE pv.created_at >= $1
              AND pv.created_at <= $2
              AND ($3 IS NULL OR pv.post_id = $3)
              AND ($4 IS NULL OR p.author_id = $4)
        ),
        bucketed AS (
            SELECT
                bucket,
                COUNT(*)::BIGINT AS views,
                COUNT(
                    DISTINCT COALESCE(
                        filtered.user_id::text,
                        CONCAT('ip:', COALESCE(filtered.ip_address, ''))
                    )
                )::BIGINT AS unique_visitors
            FROM filtered
            GROUP BY bucket
        )
        SELECT
            bucket,
            CASE WHEN $5 THEN unique_visitors ELSE views END AS views,
            unique_visitors,
            COUNT(*) OVER () AS total
        FROM bucketed
        {order_clause}
        LIMIT $6 OFFSET $7
        "#,
        bucket_expr = bucket_expr,
        order_clause = order_clause,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::Int(post_id_filter),
            Value::Int(author_id_filter),
            Value::Bool(Some(only_unique)),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
        ],
    );

    let rows = PageViewRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let data: Vec<PageViewPoint> = rows
        .into_iter()
        .map(|row| PageViewPoint {
            bucket: row.bucket,
            views: row.views,
            unique_visitors: row.unique_visitors,
        })
        .collect();

    let mut filters_obj = JsonMap::new();
    filters_obj.insert("group_by".into(), json!(interval.as_str()));
    if let Some(post_id) = post_id_filter {
        filters_obj.insert("post_id".into(), json!(post_id));
    }
    if let Some(author_id) = author_id_filter {
        filters_obj.insert("author_id".into(), json!(author_id));
    }
    if only_unique {
        filters_obj.insert("only_unique".into(), json!(true));
    }

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_filters(JsonValue::Object(filters_obj));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn comment_rate(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<CommentRateRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let min_views = request.filters.min_views.max(0);
    let sort_order = match request.filters.sort_by {
        CommentRateSort::CommentRate => "ORDER BY comment_rate DESC, comments DESC",
        CommentRateSort::Comments => "ORDER BY comments DESC, comment_rate DESC",
    };

    let sql = format!(
        r#"
        WITH view_counts AS (
            SELECT
                post_id,
                COUNT(*)::BIGINT AS views
            FROM post_views
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY post_id
        ),
        comment_counts AS (
            SELECT
                post_id,
                COUNT(*)::BIGINT AS comments
            FROM post_comments
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY post_id
        ),
        combined AS (
            SELECT
                p.id AS post_id,
                p.title,
                COALESCE(vc.views, 0) AS views,
                COALESCE(cc.comments, 0) AS comments,
                CASE
                    WHEN COALESCE(vc.views, 0) = 0 THEN 0::FLOAT8
                    ELSE ROUND(
                        (COALESCE(cc.comments, 0)::NUMERIC / vc.views::NUMERIC) * 100,
                        2
                    )::FLOAT8
                END AS comment_rate
            FROM posts p
            LEFT JOIN view_counts vc ON vc.post_id = p.id
            LEFT JOIN comment_counts cc ON cc.post_id = p.id
            WHERE COALESCE(vc.views, 0) >= $3
        )
        SELECT
            post_id,
            title,
            views,
            comments,
            comment_rate,
            COUNT(*) OVER () AS total
        FROM combined
        {sort_order}
        LIMIT $4 OFFSET $5
        "#,
        sort_order = sort_order,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::BigInt(Some(min_views as i64)),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
        ],
    );

    let rows = CommentRateRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let data: Vec<CommentRatePoint> = rows
        .into_iter()
        .map(|row| CommentRatePoint {
            post_id: row.post_id,
            title: row.title,
            views: row.views,
            comments: row.comments,
            comment_rate: row.comment_rate,
        })
        .collect();

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_sorted_by(match request.filters.sort_by {
            CommentRateSort::CommentRate => "comment_rate".to_string(),
            CommentRateSort::Comments => "comments".to_string(),
        })
        .with_filters(json!({
            "min_views": min_views,
            "sort_by": match request.filters.sort_by {
                CommentRateSort::CommentRate => "comment_rate",
                CommentRateSort::Comments => "comments",
            }
        }));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn newsletter_growth(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<NewsletterGrowthRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let interval = request.filters.group_by;
    let created_bucket_expr = interval.to_bucket_expr("created_at");
    let updated_bucket_expr = interval.to_bucket_expr("updated_at");
    let order_clause = format!("ORDER BY bucket {}", resolved.sort_order.as_sql());

    let sql = format!(
        r#"
        WITH new_subscribers AS (
            SELECT
                {created_bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS new_subscribers
            FROM newsletter_subscribers
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY 1
        ),
        confirmed AS (
            SELECT
                {updated_bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS confirmed
            FROM newsletter_subscribers
            WHERE status = 'confirmed'
              AND updated_at >= $1 AND updated_at <= $2
            GROUP BY 1
        ),
        unsubscribed AS (
            SELECT
                {updated_bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS unsubscribed
            FROM newsletter_subscribers
            WHERE status = 'unsubscribed'
              AND updated_at >= $1 AND updated_at <= $2
            GROUP BY 1
        ),
        combined AS (
            SELECT
                COALESCE(n.bucket, c.bucket, u.bucket) AS bucket,
                COALESCE(n.new_subscribers, 0) AS new_subscribers,
                COALESCE(c.confirmed, 0) AS confirmed,
                COALESCE(u.unsubscribed, 0) AS unsubscribed
            FROM new_subscribers n
            FULL OUTER JOIN confirmed c ON n.bucket = c.bucket
            FULL OUTER JOIN unsubscribed u ON COALESCE(n.bucket, c.bucket) = u.bucket
        )
        SELECT
            bucket,
            new_subscribers,
            confirmed,
            unsubscribed,
            (confirmed - unsubscribed) AS net_growth,
            COUNT(*) OVER () AS total
        FROM combined
        {order_clause}
        LIMIT $3 OFFSET $4
        "#,
        created_bucket_expr = created_bucket_expr,
        updated_bucket_expr = updated_bucket_expr,
        order_clause = order_clause,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
        ],
    );

    let rows = NewsletterGrowthRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let data: Vec<NewsletterGrowthPoint> = rows
        .into_iter()
        .map(|row| NewsletterGrowthPoint {
            bucket: row.bucket,
            new_subscribers: row.new_subscribers,
            confirmed: row.confirmed,
            unsubscribed: row.unsubscribed,
            net_growth: row.net_growth,
        })
        .collect();

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_filters(json!({ "group_by": interval.as_str() }));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn media_upload_trends(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<MediaUploadRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;
    let resolved = request.envelope.resolve();
    let limit = resolved.per_page as i64;
    let offset = resolved.offset() as i64;

    let interval = request.filters.group_by;
    let bucket_expr = interval.to_bucket_expr("created_at");
    let order_clause = format!("ORDER BY bucket {}", resolved.sort_order.as_sql());

    let sql = format!(
        r#"
        WITH bucketed AS (
            SELECT
                {bucket_expr} AS bucket,
                COUNT(*)::BIGINT AS upload_count,
                COALESCE(SUM(size), 0)::BIGINT AS total_size_bytes
            FROM media
            WHERE created_at >= $1 AND created_at <= $2
            GROUP BY 1
        )
        SELECT
            bucket,
            upload_count,
            ROUND((total_size_bytes::NUMERIC / 1024 / 1024), 2)::FLOAT8 AS total_size_mb,
            CASE
                WHEN upload_count = 0 THEN 0::FLOAT8
                ELSE ROUND(((total_size_bytes::NUMERIC / upload_count::NUMERIC) / 1024 / 1024), 2)::FLOAT8
            END AS avg_size_mb,
            COUNT(*) OVER () AS total
        FROM bucketed
        {order_clause}
        LIMIT $3 OFFSET $4
        "#,
        bucket_expr = bucket_expr,
        order_clause = order_clause,
    );

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(resolved.date_to))),
            Value::BigInt(Some(limit)),
            Value::BigInt(Some(offset)),
        ],
    );

    let rows = MediaUploadRow::find_by_statement(stmt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let total = rows
        .first()
        .and_then(|row| row.total)
        .unwrap_or_default()
        .max(0) as u64;

    let data: Vec<MediaUploadPoint> = rows
        .into_iter()
        .map(|row| MediaUploadPoint {
            bucket: row.bucket,
            upload_count: row.upload_count,
            total_size_mb: row.total_size_mb,
            avg_size_mb: row.avg_size_mb,
        })
        .collect();

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_filters(json!({ "group_by": interval.as_str() }));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
pub async fn dashboard_summary(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<DashboardSummaryRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ValidatedJson(request) = payload;

    let summary_range = resolve_dashboard_range(&request);
    let (date_from, date_to, page, per_page) = summary_range;

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT
            (SELECT COUNT(*)::BIGINT FROM users) AS users_total,
            (SELECT COUNT(*)::BIGINT FROM users WHERE created_at >= $1 AND created_at <= $2) AS users_new,
            (SELECT COUNT(*)::BIGINT FROM posts WHERE status = 'published') AS posts_published,
            (SELECT COUNT(*)::BIGINT FROM posts WHERE status = 'draft') AS posts_drafts,
            (SELECT COUNT(*)::BIGINT FROM post_views WHERE created_at >= $1 AND created_at <= $2) AS views_in_period,
            (SELECT COUNT(*)::BIGINT FROM post_comments WHERE created_at >= $1 AND created_at <= $2) AS comments_in_period,
            (SELECT COUNT(*)::BIGINT FROM newsletter_subscribers WHERE status = 'confirmed' AND updated_at >= $1 AND updated_at <= $2) AS newsletter_confirmed,
            (SELECT COUNT(*)::BIGINT FROM media) AS media_total,
            (SELECT COUNT(*)::BIGINT FROM media WHERE created_at >= $1 AND created_at <= $2) AS media_uploads
        "#,
        vec![
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(date_from))),
            Value::ChronoDateTimeWithTimeZone(Some(Box::new(date_to))),
        ],
    );

    let row = DashboardSummaryRow::find_by_statement(stmt)
        .one(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?
        .unwrap_or(DashboardSummaryRow {
            users_total: 0,
            users_new: 0,
            posts_published: 0,
            posts_drafts: 0,
            views_in_period: 0,
            comments_in_period: 0,
            newsletter_confirmed: 0,
            media_total: 0,
            media_uploads: 0,
        });

    let data = DashboardSummaryData {
        users: DashboardSummaryUsers {
            total: row.users_total,
            new_in_period: row.users_new,
        },
        posts: DashboardSummaryPosts {
            published: row.posts_published,
            drafts: row.posts_drafts,
            views_in_period: row.views_in_period,
        },
        engagement: DashboardSummaryEngagement {
            comments_in_period: row.comments_in_period,
            newsletter_confirmed: row.newsletter_confirmed,
        },
        media: DashboardSummaryMedia {
            total_files: row.media_total,
            uploads_in_period: row.media_uploads,
        },
    };

    let filters_obj = json!({
        "period": request.filters.period.as_str()
    });

    let meta = AnalyticsMeta::new(1, page, per_page)
        .with_filters(filters_obj)
        .with_interval(format!(
            "{}-{}",
            date_from.date_naive(),
            date_to.date_naive()
        ));

    Ok(Json(AnalyticsEnvelopeResponse { data, meta }))
}

fn status_array_value(statuses: Option<&Vec<PostStatus>>) -> Value {
    match statuses {
        Some(list) if !list.is_empty() => {
            let values = list
                .iter()
                .map(|status| Value::String(Some(Box::new(status.to_string()))))
                .collect::<Vec<_>>();
            Value::Array(ArrayType::String, Some(Box::new(values)))
        }
        Some(_) => Value::Array(ArrayType::String, Some(Box::new(Vec::<Value>::new()))),
        None => Value::Array(ArrayType::String, None),
    }
}

fn status_label_from_enum(status: &PostStatus) -> String {
    match status {
        PostStatus::Draft => "Draft".to_string(),
        PostStatus::Published => "Published".to_string(),
        PostStatus::Archived => "Archived".to_string(),
    }
}

fn status_label_from_str(status: &str) -> String {
    match status {
        "draft" => "Draft".to_string(),
        "published" => "Published".to_string(),
        "archived" => "Archived".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => {
                    let mut label = String::new();
                    label.push(first.to_ascii_uppercase());
                    label.extend(chars.flat_map(|c| c.to_lowercase()));
                    label
                }
                None => String::new(),
            }
        }
    }
}

fn parse_status_filters(
    statuses: Option<&Vec<String>>,
) -> Result<Option<Vec<PostStatus>>, ErrorResponse> {
    match statuses {
        None => Ok(None),
        Some(values) => {
            let mut parsed = Vec::new();
            for value in values {
                let status = match value.to_ascii_lowercase().as_str() {
                    "draft" => PostStatus::Draft,
                    "published" => PostStatus::Published,
                    "archived" => PostStatus::Archived,
                    _ => {
                        return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                            .with_message(format!("Invalid post status filter: {}", value)))
                    }
                };
                parsed.push(status);
            }
            Ok(Some(parsed))
        }
    }
}

fn resolve_dashboard_range(
    request: &DashboardSummaryRequest,
) -> (DateTimeWithTimeZone, DateTimeWithTimeZone, u64, u64) {
    if let Some(envelope) = &request.envelope {
        let resolved = envelope.resolve();
        return (
            resolved.date_from,
            resolved.date_to,
            resolved.page,
            resolved.per_page,
        );
    }

    let period = request.filters.period;
    let duration: ChronoDuration = period.as_duration();
    let today: NaiveDate = Utc::now().date_naive();
    let date_to = today;
    let date_from = today
        .checked_sub_signed(duration)
        .unwrap_or_else(|| today - ChronoDuration::days(30));

    let temp_envelope = AnalyticsEnvelope {
        date_from: Some(date_from),
        date_to: Some(date_to),
        page: Some(1),
        per_page: Some(1),
        sort_by: None,
        sort_order: None,
    };

    let resolved = temp_envelope.resolve();
    (
        resolved.date_from,
        resolved.date_to,
        resolved.page,
        resolved.per_page,
    )
}

use std::{collections::BTreeMap, ops::Bound};

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationError, ValidationErrors};

pub const DEFAULT_PER_PAGE: u64 = 30;
pub const MAX_PER_PAGE: u64 = 200;

/// Shared request envelope for analytics endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsEnvelope {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_from: Option<NaiveDate>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_to: Option<NaiveDate>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub per_page: Option<u64>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<String>,
}

impl AnalyticsEnvelope {
    pub fn resolve(&self) -> ResolvedAnalyticsEnvelope {
        let now = Utc::now().date_naive();

        let upper_bound = self.date_to.unwrap_or(now);
        let lower_bound = self.date_from.unwrap_or_else(|| {
            upper_bound
                .checked_sub_signed(Duration::days(30))
                .unwrap_or(upper_bound)
        });

        let per_page = self
            .per_page
            .map(|value| value.clamp(1, MAX_PER_PAGE))
            .unwrap_or(DEFAULT_PER_PAGE);
        let page = self.page.unwrap_or(1).max(1);

        let sort_order =
            SortOrder::from_option(self.sort_order.as_ref().map(|value| value.as_str()));

        ResolvedAnalyticsEnvelope {
            date_from: start_of_day(lower_bound),
            date_to: end_of_day(upper_bound),
            page,
            per_page,
            sort_by: self
                .sort_by
                .as_ref()
                .map(|value| value.trim().to_lowercase()),
            sort_order,
        }
    }
}

impl Validate for AnalyticsEnvelope {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if let Some(page) = self.page {
            if page == 0 {
                errors.add(
                    "page",
                    ValidationError::new("min")
                        .with_message("page must be greater than or equal to 1".into()),
                );
            }
        }

        if let Some(per_page) = self.per_page {
            if !(1..=MAX_PER_PAGE).contains(&per_page) {
                errors.add(
                    "per_page",
                    ValidationError::new("range").with_message(
                        format!("per_page must be between 1 and {}", MAX_PER_PAGE).into(),
                    ),
                );
            }
        }

        if let Some(sort_order) = &self.sort_order {
            let normalized = sort_order.trim().to_ascii_lowercase();
            if normalized != "asc" && normalized != "desc" {
                errors.add(
                    "sort_order",
                    ValidationError::new("one_of")
                        .with_message("sort_order must be 'asc' or 'desc'".into()),
                );
            }
        }

        if let (Some(from), Some(to)) = (self.date_from, self.date_to) {
            if from > to {
                errors.add(
                    "date_from",
                    ValidationError::new("lte")
                        .with_message("date_from must be before or equal to date_to".into()),
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedAnalyticsEnvelope {
    pub date_from: DateTimeWithTimeZone,
    pub date_to: DateTimeWithTimeZone,
    pub page: u64,
    pub per_page: u64,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

impl ResolvedAnalyticsEnvelope {
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    pub fn bounds(&self) -> (Bound<DateTimeWithTimeZone>, Bound<DateTimeWithTimeZone>) {
        (
            Bound::Included(self.date_from),
            Bound::Included(self.date_to),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }

    fn from_option(value: Option<&str>) -> Self {
        match value {
            Some(v) if v.eq_ignore_ascii_case("asc") => SortOrder::Asc,
            _ => SortOrder::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsInterval {
    Hour,
    Day,
    Week,
    Month,
}

impl AnalyticsInterval {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalyticsInterval::Hour => "hour",
            AnalyticsInterval::Day => "day",
            AnalyticsInterval::Week => "week",
            AnalyticsInterval::Month => "month",
        }
    }

    pub fn to_bucket_expr(&self, column: &str) -> String {
        match self {
            AnalyticsInterval::Hour => {
                format!("to_char(date_trunc('hour', {column}), 'YYYY-MM-DD HH24:00')")
            }
            AnalyticsInterval::Day => {
                format!("to_char(date_trunc('day', {column}), 'YYYY-MM-DD')")
            }
            AnalyticsInterval::Week => {
                format!("to_char(date_trunc('week', {column}), 'IYYY-\"W\"IW')")
            }
            AnalyticsInterval::Month => {
                format!("to_char(date_trunc('month', {column}), 'YYYY-MM')")
            }
        }
    }
}

impl Default for AnalyticsInterval {
    fn default() -> Self {
        AnalyticsInterval::Day
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegistrationTrendsFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for RegistrationTrendsFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationTrendsRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: RegistrationTrendsFilters,
}

impl Validate for RegistrationTrendsRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationTrendPoint {
    pub bucket: String,
    pub new_users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerificationRatesFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for VerificationRatesFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRatesRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: VerificationRatesFilters,
}

impl Validate for VerificationRatesRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationRatePoint {
    pub bucket: String,
    pub requested: i64,
    pub verified: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PublishingTrendsFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
    #[serde(default)]
    pub status: Option<Vec<String>>,
}

impl Default for PublishingTrendsFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Week,
            status: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingTrendsRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: PublishingTrendsFilters,
}

impl Validate for PublishingTrendsRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishingTrendPoint {
    pub bucket: String,
    pub counts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PageViewsFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub post_id: Option<i32>,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub author_id: Option<i32>,
    #[serde(default)]
    pub only_unique: bool,
}

impl Default for PageViewsFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
            post_id: None,
            author_id: None,
            only_unique: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageViewsRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: PageViewsFilters,
}

impl Validate for PageViewsRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageViewPoint {
    pub bucket: String,
    pub views: i64,
    pub unique_visitors: i64,
}

fn default_min_views() -> i64 {
    100
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommentRateSort {
    CommentRate,
    Comments,
}

impl Default for CommentRateSort {
    fn default() -> Self {
        CommentRateSort::CommentRate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CommentRateFilters {
    #[serde(default = "default_min_views")]
    pub min_views: i64,
    #[serde(default)]
    pub sort_by: CommentRateSort,
}

impl Default for CommentRateFilters {
    fn default() -> Self {
        Self {
            min_views: default_min_views(),
            sort_by: CommentRateSort::CommentRate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentRateRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: CommentRateFilters,
}

impl Validate for CommentRateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CommentRatePoint {
    pub post_id: i32,
    pub title: String,
    pub views: i64,
    pub comments: i64,
    pub comment_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NewsletterGrowthFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for NewsletterGrowthFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Week,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterGrowthRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: NewsletterGrowthFilters,
}

impl Validate for NewsletterGrowthRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NewsletterGrowthPoint {
    pub bucket: String,
    pub new_subscribers: i64,
    pub confirmed: i64,
    pub unsubscribed: i64,
    pub net_growth: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MediaUploadFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for MediaUploadFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: MediaUploadFilters,
}

impl Validate for MediaUploadRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaUploadPoint {
    pub bucket: String,
    pub upload_count: i64,
    pub total_size_mb: f64,
    pub avg_size_mb: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DashboardPeriod {
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
}

impl DashboardPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            DashboardPeriod::SevenDays => "7d",
            DashboardPeriod::ThirtyDays => "30d",
            DashboardPeriod::NinetyDays => "90d",
        }
    }

    pub fn as_duration(&self) -> Duration {
        match self {
            DashboardPeriod::SevenDays => Duration::days(7),
            DashboardPeriod::ThirtyDays => Duration::days(30),
            DashboardPeriod::NinetyDays => Duration::days(90),
        }
    }
}

impl Default for DashboardPeriod {
    fn default() -> Self {
        DashboardPeriod::ThirtyDays
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DashboardSummaryFilters {
    #[serde(default)]
    pub period: DashboardPeriod,
}

impl Default for DashboardSummaryFilters {
    fn default() -> Self {
        Self {
            period: DashboardPeriod::ThirtyDays,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummaryRequest {
    #[serde(flatten)]
    pub envelope: Option<AnalyticsEnvelope>,
    #[serde(default)]
    pub filters: DashboardSummaryFilters,
}

impl Validate for DashboardSummaryRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if let Some(envelope) = &self.envelope {
            envelope.validate()?;
        }
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryUsers {
    pub total: i64,
    pub new_in_period: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryPosts {
    pub published: i64,
    pub drafts: i64,
    pub views_in_period: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryEngagement {
    pub comments_in_period: i64,
    pub newsletter_confirmed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryMedia {
    pub total_files: i64,
    pub uploads_in_period: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryData {
    pub users: DashboardSummaryUsers,
    pub posts: DashboardSummaryPosts,
    pub engagement: DashboardSummaryEngagement,
    pub media: DashboardSummaryMedia,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorted_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters_applied: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl AnalyticsMeta {
    pub fn new(total: u64, page: u64, per_page: u64) -> Self {
        Self {
            total,
            page,
            per_page,
            interval: None,
            sorted_by: None,
            filters_applied: None,
            notes: None,
        }
    }

    pub fn with_interval(mut self, interval: impl Into<String>) -> Self {
        self.interval = Some(interval.into());
        self
    }

    pub fn with_sorted_by(mut self, sorted_by: impl Into<String>) -> Self {
        self.sorted_by = Some(sorted_by.into());
        self
    }

    pub fn with_filters(mut self, filters: Value) -> Self {
        self.filters_applied = Some(filters);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsEnvelopeResponse<T> {
    pub data: T,
    pub meta: AnalyticsMeta,
}

fn start_of_day(date: NaiveDate) -> DateTimeWithTimeZone {
    let offset = FixedOffset::east_opt(0).expect("UTC offset available");
    offset
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .expect("valid start-of-day datetime")
}

fn end_of_day(date: NaiveDate) -> DateTimeWithTimeZone {
    let offset = FixedOffset::east_opt(0).expect("UTC offset available");
    offset
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 23, 59, 59)
        .single()
        .expect("valid end-of-day datetime")
}

fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    match value {
        Some(raw) => parse_date(&raw)
            .map(Some)
            .map_err(|err| serde::de::Error::custom(err.to_string())),
        None => Ok(None),
    }
}

fn parse_date(value: &str) -> Result<NaiveDate, chrono::ParseError> {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(date);
    }

    DateTime::parse_from_rfc3339(value).map(|dt| dt.date_naive())
}

/analytics/v1/dashboard/summary
{
  "data": {
    "users": {
      "total": 51,
      "new_in_period": 1
    },
    "posts": {
      "published": 64,
      "drafts": 63,
      "views_in_period": 0
    },
    "engagement": {
      "comments_in_period": 0,
      "newsletter_confirmed": 0
    },
    "media": {
      "total_files": 11,
      "uploads_in_period": 11
    }
  },
  "meta": {
    "total": 1,
    "page": 1,
    "per_page": 30,
    "interval": "2025-10-07-2025-11-06",
    "filters_applied": {
      "period": "30d"
    }
  }
}

/analytics/v1/user/verification-rates
{
  "data": [
    {
      "bucket": "2025-10-31",
      "requested": 0,
      "verified": 3,
      "success_rate": 0.0
    }
  ],
  "meta": {
    "total": 1,
    "page": 1,
    "per_page": 30,
    "interval": "day",
    "sorted_by": "bucket",
    "filters_applied": {
      "group_by": "day"
    }
  }
}

/analytics/v1/engagement/comment-rate
{
  "data": [],
  "meta": {
    "total": 0,
    "page": 1,
    "per_page": 30,
    "interval": "day",
    "filters_applied": {
      "group_by": "day"
    }
  }
}

/analytics/v1/engagement/page-views
{
  "data": [],
  "meta": {
    "total": 0,
    "page": 1,
    "per_page": 30,
    "interval": "day",
    "filters_applied": {
      "group_by": "day"
    }
  }
}


/analytics/v1/content/publishing-trends
{
  "data": [
    {
      "bucket": "2025-W45",
      "counts": {
        "Draft": 8
      }
    }
  ],
  "meta": {
    "total": 1,
    "page": 1,
    "per_page": 30,
    "interval": "week",
    "sorted_by": "bucket",
    "filters_applied": {
      "group_by": "week"
    }
  }
}


/analytics/v1/engagement/page-views
{
  "data": [],
  "meta": {
    "total": 0,
    "page": 1,
    "per_page": 30,
    "interval": "day",
    "filters_applied": {
      "group_by": "day"
    }
  }
}


/analytics/v1/user/registration-trends
{
  "data": [
    {
      "bucket": "2025-10-31",
      "new_users": 1
    }
  ],
  "meta": {
    "total": 1,
    "page": 1,
    "per_page": 30,
    "interval": "day",
    "sorted_by": "bucket",
    "filters_applied": {
      "group_by": "day"
    }
  }
}
