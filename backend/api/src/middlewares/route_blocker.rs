use crate::error::RouteBlockerError;
use crate::services::route_blocker_service::RouteBlockerService;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::env;
use tracing::{debug, error, warn};

pub async fn block_routes(req: Request, next: Next) -> Result<Response, RouteBlockerError> {
    let path = req.uri().path().to_string();

    let is_development =
        env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "development";

    if is_development {
        debug!(path, "Route blocker disabled in development mode");
        return Ok(next.run(req).await);
    }

    let state = req
        .extensions()
        .get::<crate::state::AppState>()
        .cloned()
        .ok_or_else(|| {
            error!(path, "App state missing from request extensions");
            RouteBlockerError::CheckFailed("Application state unavailable".to_string())
        })?;

    match RouteBlockerService::is_route_blocked(State(state), &path).await {
        Ok(true) => {
            warn!(path, "Route blocked by dynamic route_blocker middleware");
            return Err(RouteBlockerError::Blocked { path });
        }
        Ok(false) => {
            debug!(path, "Route allowed");
        }
        Err(e) => {
            error!(error = %e, path, "Failed to check route status");
            return Err(RouteBlockerError::CheckFailed(e.to_string()));
        }
    }

    Ok(next.run(req).await)
}
