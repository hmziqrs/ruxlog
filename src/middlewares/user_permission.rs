use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{debug, instrument, warn};

use crate::{
    db::sea_models::user::{self, UserRole},
    error::{ErrorCode, ErrorResponse},
    services::auth::AuthSession,
};

#[instrument(skip(user), fields(user_id, user_role, required_role = ?req_role, result))]
fn check_user_role(user: Option<user::Model>, req_role: UserRole) -> Result<bool, Response> {
    let user = user.ok_or_else(|| {
        warn!(required_role = ?req_role, "No authenticated user");
        tracing::Span::current().record("result", "no_user");
        ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Unauthorized")
            .into_response()
    })?;

    tracing::Span::current().record("user_id", user.id);
    tracing::Span::current().record("user_role", user.role.to_string());

    if user.role.to_i32() >= req_role.to_i32() {
        debug!(
            user_id = user.id,
            user_role = user.role.to_string(),
            required_role = ?req_role,
            "Permission check passed"
        );
        tracing::Span::current().record("result", "allowed");
        Ok(true)
    } else {
        warn!(
            user_id = user.id,
            user_role = user.role.to_string(),
            required_role = ?req_role,
            "Permission check failed - insufficient role"
        );
        tracing::Span::current().record("result", "denied");
        Err(ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("You don't have the required permission level")
            .into_response())
    }
}

#[instrument(skip(auth, request, next), fields(required_role = "user"))]
pub async fn user(auth: AuthSession, request: Request, next: Next) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::User)?;
    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "author"))]
pub async fn author(auth: AuthSession, request: Request, next: Next) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::Author)?;
    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "moderator"))]
pub async fn moderator(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::Moderator)?;
    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "admin"))]
pub async fn admin(auth: AuthSession, request: Request, next: Next) -> Result<Response, Response> {
    let user_opt = auth.user.clone();
    check_user_role(user_opt.clone(), UserRole::Admin)?;

    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "super_admin"))]
pub async fn super_admin(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    check_user_role(auth.user, UserRole::SuperAdmin)?;
    Ok(next.run(request).await)
}
