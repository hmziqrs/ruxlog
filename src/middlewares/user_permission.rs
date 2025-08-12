use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    db::sea_models::user::{self, UserRole},
    error::{ErrorCode, ErrorResponse},
    services::auth::AuthSession,
};

fn check_user_role(user: Option<user::Model>, req_role: UserRole) -> Result<bool, Response> {
    let user = user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Unauthorized")
            .into_response()
    })?;

    if user.role.to_i32() >= req_role.to_i32() {
        Ok(true)
    } else {
        Err(ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("You don't have the required permission level")
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
    let user_opt = auth.user.clone();
    check_user_role(user_opt.clone(), UserRole::Admin)?;

    // Allow admin to access 2FA setup without requiring 2FA to be already enabled
    let path = request.uri().path();
    if !path.contains("/2fa/setup") {
        if let Some(user) = user_opt {
            if !user.two_fa_enabled {
                return Ok((
                    axum::http::StatusCode::FORBIDDEN,
                    axum::Json(
                        serde_json::json!({ "message": "Two-factor authentication required" }),
                    ),
                )
                    .into_response());
            }
        }
    }

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
