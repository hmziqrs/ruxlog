use axum::{extract::State, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info, instrument};

use crate::{
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    modules::observability_v1::validator::{
        V1ErrorStatsPayload, V1LatencyStatsPayload, V1LogsRecentPayload, V1LogsSearchPayload,
        V1MetricsSummaryPayload,
    },
    AppState,
};

#[debug_handler]
#[instrument(skip(state))]
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    let status = if client.is_enabled() {
        json!({
            "observability": "enabled",
            "backend": "openobserve"
        })
    } else {
        json!({
            "observability": "disabled",
            "message": "Set OTEL_EXPORTER_OTLP_ENDPOINT to enable"
        })
    };

    Ok(Json(status))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn search_logs(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1LogsSearchPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("OpenObserve is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let sql = payload.get_sql();
    let stream = payload.get_stream();
    let from = payload.get_from();
    let size = payload.get_size();

    info!(
        stream = %stream,
        sql = %sql,
        from = from,
        size = size,
        "Searching logs"
    );

    let response = client
        .search(&stream, &sql, start_time, end_time, from, size)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to search logs");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query observability data")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.total,
        "from": response.from,
        "size": response.size,
        "took_ms": response.took,
        "scan_size_mb": response.scan_size
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn recent_logs(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1LogsRecentPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("OpenObserve is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let sql = payload.build_sql();
    let limit = payload.get_limit();

    info!(
        sql = %sql,
        limit = limit,
        "Fetching recent logs"
    );

    let response = client
        .search("default", &sql, start_time, end_time, 0, limit)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch recent logs");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query recent logs")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.total,
        "took_ms": response.took
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn metrics_summary(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1MetricsSummaryPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("OpenObserve is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let sql = payload.build_sql();

    info!(sql = %sql, "Fetching metrics summary");

    let response = client
        .search("default", &sql, start_time, end_time, 0, 1000)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch metrics");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query metrics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.total,
        "took_ms": response.took
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn error_stats(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1ErrorStatsPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("OpenObserve is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let sql = payload.build_sql();

    info!(sql = %sql, "Fetching error statistics");

    let response = client
        .search("default", &sql, start_time, end_time, 0, 100)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch error stats");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query error statistics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.total,
        "took_ms": response.took
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn latency_stats(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1LatencyStatsPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("OpenObserve is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let sql = payload.build_sql();

    info!(sql = %sql, "Fetching latency statistics");

    let response = client
        .search("default", &sql, start_time, end_time, 0, 100)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch latency stats");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query latency statistics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.total,
        "took_ms": response.took
    })))
}

#[debug_handler]
#[instrument(skip(state))]
pub async fn auth_stats(State(state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.openobserve_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("OpenObserve is not configured"));
    }

    let now = chrono::Utc::now();
    let start_time = (now - chrono::Duration::hours(24)).timestamp_micros();
    let end_time = now.timestamp_micros();

    let sql = "SELECT event_type, COUNT(*) AS count \
               FROM {stream} \
               WHERE event_type LIKE 'auth.%' \
               GROUP BY event_type \
               ORDER BY count DESC";

    info!("Fetching authentication statistics");

    let response = client
        .search("default", sql, start_time, end_time, 0, 100)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch auth stats");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query authentication statistics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.total,
        "took_ms": response.took
    })))
}
