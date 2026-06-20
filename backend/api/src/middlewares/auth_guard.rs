//! Authentication guard middleware functions using rux-auth
//!
//! Uses composable requirements - single middleware per route with chained checks.

use axum::{extract::Request, middleware::Next, response::Response, Extension};
use rux_auth::{auth_requirements, check_requirements, AuthError, AuthSession};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

use crate::services::auth::AuthBackend;

/// Helper to create AuthSession from DB + Redis extensions and a Session.
///
/// V-HIGH-2: `AuthBackend` requires the Redis pool so the per-request
/// `is_session_revoked` `SISMEMBER` can run. The router layers BOTH a
/// `DatabaseConnection` extension and a `RedisPool` extension
/// (see `main.rs`); every guard below consumes both.
async fn make_auth_session(
    db: &DatabaseConnection,
    redis_pool: RedisPool,
    session: Session,
) -> AuthSession<AuthBackend> {
    let backend = AuthBackend::new(db, redis_pool);
    AuthSession::new(backend, session).await
}

// Role level constants matching user::UserRole::to_i32()
pub const ROLE_USER: i32 = 0;
pub const ROLE_AUTHOR: i32 = 1;
pub const ROLE_MODERATOR: i32 = 2;
pub const ROLE_ADMIN: i32 = 3;
pub const ROLE_SUPER_ADMIN: i32 = 4;

// =============================================================================
// Single-purpose guards (for simple cases)
// =============================================================================

/// Require user to be authenticated only
pub async fn authenticated(
    Extension(db): Extension<DatabaseConnection>,
    Extension(redis_pool): Extension<RedisPool>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let mut auth = make_auth_session(&db, redis_pool, session).await;
    check_requirements(&mut auth, &auth_requirements().authenticated().not_banned()).await?;
    Ok(next.run(request).await)
}

/// Require user to NOT be authenticated (for login/register routes)
pub async fn unauthenticated(
    Extension(db): Extension<DatabaseConnection>,
    Extension(redis_pool): Extension<RedisPool>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let mut auth = make_auth_session(&db, redis_pool, session).await;
    check_requirements(&mut auth, &auth_requirements().unauthenticated()).await?;
    Ok(next.run(request).await)
}

/// Require user to be authenticated but NOT verified (for verification routes)
pub async fn unverified(
    Extension(db): Extension<DatabaseConnection>,
    Extension(redis_pool): Extension<RedisPool>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let mut auth = make_auth_session(&db, redis_pool, session).await;
    check_requirements(&mut auth, &auth_requirements().authenticated().unverified()).await?;
    Ok(next.run(request).await)
}

// =============================================================================
// Composed guards (authenticated + verified + role in one middleware)
// =============================================================================

/// Require authenticated + verified user
pub async fn verified(
    Extension(db): Extension<DatabaseConnection>,
    Extension(redis_pool): Extension<RedisPool>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let mut auth = make_auth_session(&db, redis_pool, session).await;
    check_requirements(
        &mut auth,
        &auth_requirements().authenticated().verified().not_banned(),
    )
    .await?;
    Ok(next.run(request).await)
}

/// Require authenticated + verified + minimum role (single middleware)
pub async fn verified_with_role<const LEVEL: i32>(
    Extension(db): Extension<DatabaseConnection>,
    Extension(redis_pool): Extension<RedisPool>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let mut auth = make_auth_session(&db, redis_pool, session).await;
    check_requirements(
        &mut auth,
        &auth_requirements()
            .authenticated()
            .verified()
            .not_banned()
            .role_min(LEVEL),
    )
    .await?;
    Ok(next.run(request).await)
}
