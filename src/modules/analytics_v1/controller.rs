use std::collections::{BTreeMap, HashMap};

use axum::{extract::State, response::IntoResponse, Json};
use axum_macros::debug_handler;
use sea_orm::{
    sea_query::{ArrayType, Value},
    DatabaseBackend, FromQueryResult, Statement,
};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use tracing::instrument;

use crate::{
    db::sea_models::post::PostStatus, error::ErrorResponse, extractors::ValidatedJson,
    services::auth::AuthSession, AppState,
};

use super::validator::{
    AnalyticsEnvelopeResponse, AnalyticsMeta, PublishingTrendPoint, PublishingTrendsRequest,
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

    let filters_value = json!({ "group_by": interval.as_str() });

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_sorted_by(sort_field.to_string())
        .with_filters(filters_value);

    let response = AnalyticsEnvelopeResponse { data, meta };

    Ok(Json(response))
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

    let filters_value = json!({ "group_by": interval.as_str() });

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_sorted_by(order_target.to_string())
        .with_filters(filters_value);

    let response = AnalyticsEnvelopeResponse { data, meta };

    Ok(Json(response))
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

    let status_filter = request.filters.status.clone();
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

    for row in rows.iter() {
        let entry = bucket_counts.entry(row.bucket.clone()).or_insert_with(|| {
            ordered_buckets.push(row.bucket.clone());
            BTreeMap::new()
        });
        entry.insert(status_label_from_str(&row.status), row.count);
    }

    let mut data: Vec<PublishingTrendPoint> = Vec::new();

    for bucket in ordered_buckets {
        if let Some(counts) = bucket_counts.get(&bucket) {
            let filtered_counts = if let Some(labels) = &requested_labels {
                let mut map = BTreeMap::new();
                for label in labels {
                    let value = *counts.get(label).unwrap_or(&0);
                    map.insert(label.clone(), value);
                }
                map
            } else {
                counts.clone()
            };

            if has_status_filter && filtered_counts.values().all(|&value| value == 0) {
                continue;
            }

            data.push(PublishingTrendPoint {
                bucket,
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

    let response = AnalyticsEnvelopeResponse { data, meta };

    Ok(Json(response))
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
