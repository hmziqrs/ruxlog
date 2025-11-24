use crate::db::sea_models::route_status::Entity as RouteStatus;
use crate::error::ErrorResponse;
use crate::state::AppState;
use axum::extract::State;
use serde_json::json;
use std::error::Error;
use tower_sessions_redis_store::fred::prelude::*;
use tracing::{debug, info};

pub struct RouteBlockerService;

impl RouteBlockerService {
    pub const BLOCKED_ROUTES_KEY: &'static str = "blocked_routes";
    pub const KNOWN_ROUTES_KEY: &'static str = "known_routes";

    pub async fn record_route_pattern(
        state: &AppState,
        pattern: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let already_cached: bool = state
            .redis_pool
            .sismember(Self::KNOWN_ROUTES_KEY, pattern)
            .await?;

        if already_cached {
            debug!(pattern, "Route pattern already cached in known_routes set");
            return Ok(());
        }

        RouteStatus::ensure_exists(&state.sea_db, pattern)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        state
            .redis_pool
            .sadd::<(), _, _>(Self::KNOWN_ROUTES_KEY, pattern)
            .await?;

        info!(pattern, "Recorded route pattern in valkey known_routes set");

        Ok(())
    }

    pub async fn is_route_blocked(
        State(state): State<AppState>,
        path: &str,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let is_blocked: bool = state
            .redis_pool
            .sismember(Self::BLOCKED_ROUTES_KEY, path)
            .await?;

        Ok(is_blocked)
    }

    pub async fn block_route(
        State(state): State<AppState>,
        pattern: String,
        reason: Option<String>,
    ) -> Result<serde_json::Value, ErrorResponse> {
        let route = RouteStatus::create_or_update(&state.sea_db, pattern.clone(), true, reason)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })?;

        Self::sync_route_to_redis(&state, &pattern, true)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })?;

        Ok(json!(route))
    }

    pub async fn unblock_route(
        State(state): State<AppState>,
        pattern: String,
    ) -> Result<serde_json::Value, ErrorResponse> {
        let route = RouteStatus::create_or_update(&state.sea_db, pattern.clone(), false, None)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })?;

        Self::sync_route_to_redis(&state, &pattern, false)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })?;

        Ok(json!(route))
    }

    pub async fn delete_route(
        State(state): State<AppState>,
        pattern: String,
    ) -> Result<serde_json::Value, ErrorResponse> {
        RouteStatus::delete_by_pattern(&state.sea_db, &pattern)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })?;

        Self::remove_route_from_redis(&state, &pattern)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })?;

        Ok(json!({ "message": "Route deleted successfully" }))
    }

    pub async fn list_blocked_routes(
        State(state): State<AppState>,
    ) -> Result<Vec<crate::db::sea_models::route_status::Model>, ErrorResponse> {
        RouteStatus::find_blocked_routes(&state.sea_db)
            .await
            .map_err(|e| {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(e.to_string())
            })
    }

    pub async fn sync_all_routes_to_redis(
        State(state): State<AppState>,
    ) -> Result<serde_json::Value, ErrorResponse> {
        RouteStatus::sync_all_to_redis(
            &state.sea_db,
            state.redis_pool,
            Self::KNOWN_ROUTES_KEY,
            Self::BLOCKED_ROUTES_KEY,
        )
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if let Some(redis_err) = err_str.strip_prefix("Redis error: ") {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(format!("Redis sync failed: {}", redis_err))
            } else {
                ErrorResponse::new(crate::error::ErrorCode::InternalServerError)
                    .with_message(format!("Database sync failed: {}", err_str))
            }
        })?;

        Ok(json!({ "message": "All routes synced to Redis successfully" }))
    }

    async fn sync_route_to_redis(
        state: &AppState,
        pattern: &str,
        is_blocked: bool,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state
            .redis_pool
            .sadd::<(), _, _>(Self::KNOWN_ROUTES_KEY, pattern)
            .await?;

        if is_blocked {
            info!(pattern, "Adding route to blocked_routes set in valkey");
            state
                .redis_pool
                .sadd::<(), _, _>(Self::BLOCKED_ROUTES_KEY, pattern)
                .await?;
        } else {
            info!(pattern, "Removing route from blocked_routes set in valkey");
            state
                .redis_pool
                .srem::<(), _, _>(Self::BLOCKED_ROUTES_KEY, pattern)
                .await?;
        }

        Ok(())
    }

    async fn remove_route_from_redis(
        state: &AppState,
        pattern: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state
            .redis_pool
            .srem::<(), _, _>(Self::BLOCKED_ROUTES_KEY, pattern)
            .await?;
        state
            .redis_pool
            .srem::<(), _, _>(Self::KNOWN_ROUTES_KEY, pattern)
            .await?;
        Ok(())
    }

    pub async fn initialize_redis_sync(
        state: &AppState,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match Self::sync_all_routes_to_redis(State(state.clone())).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("Failed to initialize Redis sync: {}", e);
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Redis sync failed: {}", e),
                )))
            }
        }
    }
}
