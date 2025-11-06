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
