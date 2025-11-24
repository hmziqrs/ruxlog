use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info, instrument};

use crate::{
    error::ErrorResponse,
    extractors::ValidatedJson,
    services::auth::AuthSession,
    services::{route_blocker_config, route_blocker_service::RouteBlockerService},
    AppState,
};

use super::validator::{V1BlockRoutePayload, V1UpdateRoutePayload, V1UpdateSyncIntervalPayload};

#[debug_handler]
#[instrument(skip(state, _auth, payload), fields(pattern))]
pub async fn block_route(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1BlockRoutePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pattern = &payload.pattern;
    tracing::Span::current().record("pattern", pattern.as_str());

    let result = RouteBlockerService::block_route(
        State(state),
        payload.pattern.clone(),
        payload.reason.clone(),
    )
    .await;

    match result {
        Ok(route) => {
            info!(pattern = %pattern, "Route blocked successfully");
            Ok((StatusCode::CREATED, Json(json!(route))))
        }
        Err(err) => {
            error!(pattern = %pattern, error = %err, "Failed to block route");
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth), fields(pattern))]
pub async fn unblock_route(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(pattern): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    tracing::Span::current().record("pattern", pattern.as_str());

    let result = RouteBlockerService::unblock_route(State(state), pattern.clone()).await;

    match result {
        Ok(route) => {
            info!(pattern = %pattern, "Route unblocked successfully");
            Ok(Json(json!(route)))
        }
        Err(err) => {
            error!(pattern = %pattern, error = %err, "Failed to unblock route");
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth, payload), fields(pattern, is_blocked = payload.is_blocked))]
pub async fn update_route_status(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(pattern): Path<String>,
    payload: ValidatedJson<V1UpdateRoutePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    tracing::Span::current().record("pattern", pattern.as_str());

    let result = if payload.is_blocked {
        RouteBlockerService::block_route(State(state), pattern.clone(), payload.reason.clone())
            .await
    } else {
        RouteBlockerService::unblock_route(State(state), pattern.clone()).await
    };

    match result {
        Ok(route) => {
            info!(
                pattern = %pattern,
                is_blocked = payload.is_blocked,
                "Route status updated successfully"
            );
            Ok(Json(json!(route)))
        }
        Err(err) => {
            error!(
                pattern = %pattern,
                is_blocked = payload.is_blocked,
                error = %err,
                "Failed to update route status"
            );
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth), fields(pattern))]
pub async fn delete_route(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(pattern): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    tracing::Span::current().record("pattern", pattern.as_str());

    let result = RouteBlockerService::delete_route(State(state), pattern.clone()).await;

    match result {
        Ok(response) => {
            info!(pattern = %pattern, "Route deleted successfully");
            Ok(Json(response))
        }
        Err(err) => {
            error!(pattern = %pattern, error = %err, "Failed to delete route");
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth))]
pub async fn list_blocked_routes(
    State(state): State<AppState>,
    _auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = RouteBlockerService::list_blocked_routes(State(state)).await;

    match result {
        Ok(routes) => {
            info!(count = routes.len(), "Retrieved blocked routes list");
            Ok(Json(json!({
                "data": routes,
                "total": routes.len(),
                "page": 1,
                "per_page": routes.len()
            })))
        }
        Err(err) => {
            error!(error = %err, "Failed to retrieve blocked routes");
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth))]
pub async fn sync_routes_to_redis(
    State(state): State<AppState>,
    _auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = RouteBlockerService::sync_all_routes_to_redis(State(state)).await;

    match result {
        Ok(response) => {
            info!("Successfully synced all routes to Redis");
            Ok(Json(response))
        }
        Err(err) => {
            error!(error = %err, "Failed to sync routes to Redis");
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(_auth))]
pub async fn get_sync_interval(_auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    let interval_secs = route_blocker_config::get_sync_interval_secs();
    info!(interval_secs, "Retrieved route blocker sync interval");
    Ok(Json(json!({ "interval_secs": interval_secs })))
}

#[debug_handler]
#[instrument(skip(_auth, payload))]
pub async fn update_sync_interval(
    _auth: AuthSession,
    payload: ValidatedJson<V1UpdateSyncIntervalPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let interval_secs = payload.interval_secs;
    route_blocker_config::set_sync_interval_secs(interval_secs);
    route_blocker_config::request_immediate_sync();
    info!(interval_secs, "Updated route blocker sync interval");

    Ok((
        StatusCode::OK,
        Json(json!({ "interval_secs": route_blocker_config::get_sync_interval_secs() })),
    ))
}

#[debug_handler]
#[instrument(skip(_auth))]
pub async fn pause_sync_interval(_auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    route_blocker_config::pause_sync();
    info!("Paused route blocker sync loop");
    Ok((
        StatusCode::OK,
        Json(json!({
            "paused": true,
            "interval_secs": route_blocker_config::get_sync_interval_secs()
        })),
    ))
}

#[debug_handler]
#[instrument(skip(_auth))]
pub async fn resume_sync_interval(_auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    route_blocker_config::resume_sync();
    info!("Resumed route blocker sync loop");
    Ok((
        StatusCode::OK,
        Json(json!({
            "paused": route_blocker_config::is_paused(),
            "interval_secs": route_blocker_config::get_sync_interval_secs()
        })),
    ))
}

#[debug_handler]
#[instrument(skip(_auth))]
pub async fn restart_sync_interval(_auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    // Ensure the loop is active and trigger an immediate sync.
    route_blocker_config::resume_sync();
    route_blocker_config::request_immediate_sync();
    info!("Restarted route blocker sync loop");
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "paused": route_blocker_config::is_paused(),
            "interval_secs": route_blocker_config::get_sync_interval_secs()
        })),
    ))
}
