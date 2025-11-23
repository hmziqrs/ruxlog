use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{debug, instrument, warn};

use crate::{
    db::sea_models::user::{self, UserRole},
    error::PermissionError,
    services::auth::AuthSession,
};

#[instrument(skip(user), fields(user_id, user_role, required_role = ?req_role, result))]
fn check_user_role(user: Option<user::Model>, req_role: UserRole) -> Result<(), PermissionError> {
    let user = user.ok_or_else(|| {
        warn!(required_role = ?req_role, "No authenticated user");
        tracing::Span::current().record("result", "no_user");
        PermissionError::NoUser
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
        Ok(())
    } else {
        warn!(
            user_id = user.id,
            user_role = user.role.to_string(),
            required_role = ?req_role,
            "Permission check failed - insufficient role"
        );
        tracing::Span::current().record("result", "denied");
        Err(PermissionError::InsufficientRole {
            required: req_role,
            actual: user.role,
        })
    }
}

#[instrument(skip(auth, request, next), fields(required_role = "user"))]
pub async fn user(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, PermissionError> {
    check_user_role(auth.user.clone(), UserRole::User)?;
    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "author"))]
pub async fn author(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, PermissionError> {
    check_user_role(auth.user.clone(), UserRole::Author)?;
    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "moderator"))]
pub async fn moderator(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, PermissionError> {
    check_user_role(auth.user.clone(), UserRole::Moderator)?;
    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "admin"))]
pub async fn admin(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, PermissionError> {
    check_user_role(auth.user.clone(), UserRole::Admin)?;

    Ok(next.run(request).await)
}
#[instrument(skip(auth, request, next), fields(required_role = "super_admin"))]
pub async fn super_admin(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, PermissionError> {
    check_user_role(auth.user.clone(), UserRole::SuperAdmin)?;
    Ok(next.run(request).await)
}
