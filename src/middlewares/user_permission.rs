use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    db::models::user::{User, UserRole},
    services::auth::AuthSession,
};

fn check_user_role(user: Option<User>, req_role: UserRole) -> Result<bool, Response> {
    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"message": "Unauthorized"})),
        )
            .into_response()
    })?;

    if user.role.to_i32() >= req_role.to_i32() {
        Ok(true)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"message": "Unauthorized"})),
        )
            .into_response())
    }
}

pub async fn user(auth: AuthSession, request: Request, next: Next) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::User)?;
    Ok(next.run(request).await)
}
pub async fn author(auth: AuthSession, request: Request, next: Next) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::Author)?;
    Ok(next.run(request).await)
}
pub async fn moderator(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::Moderator)?;
    Ok(next.run(request).await)
}
pub async fn admin(auth: AuthSession, request: Request, next: Next) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::Admin)?;
    Ok(next.run(request).await)
}
pub async fn super_admin(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::SuperAdmin)?;
    Ok(next.run(request).await)
}
