use crate::error::{ErrorCode, ErrorResponse};
use crate::services::route_blocker_service::RouteBlockerService;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::env;
use tracing::{debug, warn, error};

pub async fn block_routes(
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = req.uri().path().to_string();

    let is_development =
        env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "development";

    if is_development {
        debug!(path, "Route blocker disabled in development mode");
        return Ok(next.run(req).await);
    }

    let state = req.extensions().get::<crate::state::AppState>().unwrap();

    match RouteBlockerService::is_route_blocked(State(state.clone()), &path).await {
        Ok(true) => {
            warn!(
                path,
                "Route blocked by dynamic route_blocker middleware"
            );
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("This route is currently unavailable")
                .into_response());
        }
        Ok(false) => {
            debug!(path, "Route allowed");
        }
        Err(e) => {
            error!(error = %e, path, "Failed to check route status, allowing by default");
        }
    }

    Ok(next.run(req).await)
}
