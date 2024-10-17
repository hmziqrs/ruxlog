use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::services::auth::AuthSession;

pub async fn only_verified(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if let Some(user) = auth.user {
        if !user.is_verified {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({"message": "User not verified"})),
            )
                .into_response());
        }
    }
    Ok(next.run(request).await)
}
pub async fn only_unverified(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if let Some(user) = auth.user {
        if user.is_verified {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({"message": "Resource not available"})),
            )
                .into_response());
        }
    }
    Ok(next.run(request).await)
}

pub async fn only_unauthenticated(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if !auth.user.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Resource not available"})),
        )
            .into_response());
    }
    Ok(next.run(request).await)
}

pub async fn only_authenticated(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if auth.user.is_none() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Resource not available"})),
        )
            .into_response());
    }
    Ok(next.run(request).await)
}
