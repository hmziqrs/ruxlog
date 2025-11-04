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
    services::route_blocker_service::RouteBlockerService,
    AppState,
};

use super::validator::{V1BlockRoutePayload, V1UpdateRoutePayload};

#[debug_handler]
#[instrument(skip(state))]
pub async fn block_route(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1BlockRoutePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pattern = &payload.pattern;
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
#[instrument(skip(state))]
pub async fn unblock_route(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(pattern): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
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
#[instrument(skip(state))]
pub async fn update_route_status(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(pattern): Path<String>,
    payload: ValidatedJson<V1UpdateRoutePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = if payload.is_blocked {
        RouteBlockerService::block_route(State(state), pattern.clone(), payload.reason.clone()).await
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
#[instrument(skip(state))]
pub async fn delete_route(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(pattern): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
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
#[instrument(skip(state))]
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
                "total": routes.len()
            })))
        }
        Err(err) => {
            error!(error = %err, "Failed to retrieve blocked routes");
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state))]
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