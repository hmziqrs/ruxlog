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
    let client = &state.quickwit_client;

    let status = if client.is_enabled() {
        json!({
            "observability": "enabled",
            "backend": "quickwit",
            "index": client.logs_index()
        })
    } else {
        json!({
            "observability": "disabled",
            "message": "Set ENABLE_QUICKWIT_OTEL=true and QUICKWIT_API_URL to enable"
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
    let client = &state.quickwit_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("Quickwit is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let query = payload.get_query();
    let index = payload.get_index();
    let from = payload.get_from();
    let size = payload.get_size();

    info!(
        index = %index,
        query = %query,
        from = from,
        size = size,
        "Searching logs"
    );

    let response = client
        .search(Some(&index), &query, start_time, end_time, from, size)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to search logs in Quickwit");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query observability data")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.num_hits,
        "from": from,
        "size": size,
        "took_ms": response.elapsed_time_micros as f64 / 1000.0
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn recent_logs(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1LogsRecentPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.quickwit_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("Quickwit is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let query = payload.build_query();
    let limit = payload.get_limit();

    info!(
        query = %query,
        limit = limit,
        "Fetching recent logs"
    );

    let response = client
        .search(None, &query, start_time, end_time, 0, limit)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch recent logs from Quickwit");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query recent logs")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.num_hits,
        "took_ms": response.elapsed_time_micros as f64 / 1000.0
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn metrics_summary(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1MetricsSummaryPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.quickwit_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("Quickwit is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let query = payload.build_query();

    info!(query = %query, "Fetching metrics summary");

    let response = client
        .search(None, &query, start_time, end_time, 0, 500)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch metrics from Quickwit");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query metrics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.num_hits,
        "took_ms": response.elapsed_time_micros as f64 / 1000.0
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn error_stats(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1ErrorStatsPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.quickwit_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("Quickwit is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let query = payload.build_query();

    info!(query = %query, "Fetching error statistics");

    let response = client
        .search(
            None,
            &query,
            start_time,
            end_time,
            0,
            payload.top_n.unwrap_or(20),
        )
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch error stats from Quickwit");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query error statistics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.num_hits,
        "took_ms": response.elapsed_time_micros as f64 / 1000.0
    })))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn latency_stats(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<V1LatencyStatsPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.quickwit_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("Quickwit is not configured"));
    }

    let (start_time, end_time) = payload.get_time_range();
    let query = payload.build_query();

    info!(query = %query, "Fetching latency statistics");

    let response = client
        .search(None, &query, start_time, end_time, 0, 100)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch latency stats from Quickwit");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query latency statistics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.num_hits,
        "took_ms": response.elapsed_time_micros as f64 / 1000.0
    })))
}

#[debug_handler]
#[instrument(skip(state))]
pub async fn auth_stats(State(state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    let client = &state.quickwit_client;

    if !client.is_enabled() {
        return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
            .with_message("Quickwit is not configured"));
    }

    let now = chrono::Utc::now();
    let start_time = (now - chrono::Duration::hours(24)).timestamp_micros();
    let end_time = now.timestamp_micros();

    let query = "event_type:auth.*";

    info!("Fetching authentication statistics");

    let response = client
        .search(None, query, start_time, end_time, 0, 100)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch auth stats from Quickwit");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to query authentication statistics")
        })?;

    Ok(Json(json!({
        "data": response.hits,
        "total": response.num_hits,
        "took_ms": response.elapsed_time_micros as f64 / 1000.0
    })))
}
