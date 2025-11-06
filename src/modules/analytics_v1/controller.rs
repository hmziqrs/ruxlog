use axum::{extract::State, response::IntoResponse, Json};
use axum_macros::debug_handler;
use sea_orm::{sea_query::Value, DatabaseBackend, FromQueryResult, Statement};
use serde_json::json;
use tracing::instrument;

use crate::{
    error::ErrorResponse, extractors::ValidatedJson, services::auth::AuthSession, AppState,
};

use super::validator::{
    AnalyticsEnvelopeResponse, AnalyticsMeta, RegistrationTrendPoint, RegistrationTrendsRequest,
};

#[derive(Debug, FromQueryResult)]
struct RegistrationTrendRow {
    bucket: String,
    new_users: i64,
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

    let meta = AnalyticsMeta::new(total, resolved.page, resolved.per_page)
        .with_interval(interval.as_str().to_string())
        .with_sorted_by(sort_field.to_string())
        .with_filters(json!({ "group_by": interval.as_str() }));

    let response = AnalyticsEnvelopeResponse { data, meta };

    Ok(Json(response))
}
