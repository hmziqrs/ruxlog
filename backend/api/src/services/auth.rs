use async_trait::async_trait;
use password_auth::verify_password;
use rux_auth::{
    AuthBackend as RuxAuthBackend, AuthError, AuthErrorCode, AuthUser, BanStatus,
    SessionRevocation,
};
use sea_orm::DatabaseConnection;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, instrument, warn};

use crate::{db::sea_models::user, db::sea_models::user_ban, utils::telemetry};

/// Re-export the AuthSession from rux-auth
pub type AuthSession = rux_auth::AuthSession<AuthBackend>;

/// Authentication backend implementation
#[derive(Clone)]
pub struct AuthBackend {
    pub pool: DatabaseConnection,
}

impl AuthBackend {
    pub fn new(pool: &DatabaseConnection) -> Self {
        Self { pool: pool.clone() }
    }

    /// Revoke a live tower-sessions record from Redis by its store key.
    ///
    /// V-HIGH-2: stamping `user_sessions.revoked_at` does NOT touch the
    /// tower-sessions Redis record, so the cookie keeps authenticating until its
    /// 14-day inactivity expiry. This deletes the store key so the very next
    /// request carrying that cookie finds no session and is unauthenticated.
    ///
    /// `tower_session_id` is the `tower_sessions::session::Id` `Display` output
    /// (a 22-char base64url-no-pad i128) — the exact key `RedisStore` saves
    /// under. We `sadd` it to a revocation set as well, as defense-in-depth for
    /// any extractor that consults [`SessionRevocation`] (a future Redis-aware
    /// backend) and as an audit record if the `DEL` races a concurrent save.
    #[instrument(skip(self, redis_pool))]
    pub async fn delete_tower_session(
        &self,
        redis_pool: &tower_sessions_redis_store::fred::prelude::Pool,
        tower_session_id: &str,
    ) {
        use tower_sessions_redis_store::fred::interfaces::{KeysInterface, SetsInterface};

        // 1. Kill the live record: the next request with this cookie loads
        //    nothing from the store and is treated as anonymous.
        let del_result: Result<i64, _> = redis_pool.del(tower_session_id.to_string()).await;
        match del_result {
            Ok(n) => info!(deleted = n, "Deleted tower-session from Redis"),
            Err(e) => warn!(
                error = %e,
                "Failed to DEL tower-session from Redis (revoked_at audit row still set)"
            ),
        }

        // 2. Defense-in-depth: record the revocation so an extractor-level
        //    check (see rux_auth::SessionRevocation) can still catch it even if
        //    a concurrent session save re-created the key. TTL matches the
        //    session max-age (14 days) so the set self-cleans.
        let revoked_key = revoked_set_key();
        let sadd_result: Result<i64, _> =
            redis_pool.sadd(revoked_key.clone(), vec![tower_session_id.to_string()]).await;
        match sadd_result {
            Ok(_) => {}
            Err(e) => warn!(error = %e, "Failed to record revocation in Redis set"),
        }
        // Best-effort TTL refresh; ignore errors (key may already be gone).
        let _: Result<(), _> = redis_pool
            .expire(revoked_key, SESSION_MAX_AGE_SECS, None)
            .await;
    }

    /// Verify password against hash
    pub fn check_password(password: String, hash: &str) -> Result<bool, AuthError> {
        verify_password(password, hash)
            .map(|_| true)
            .map_err(|_| AuthError::new(AuthErrorCode::InvalidCredentials))
    }

    /// Authenticate with email and password
    #[instrument(skip(self, password), fields(email = %email, result))]
    pub async fn authenticate_password(
        &self,
        email: String,
        password: String,
    ) -> Result<Option<user::Model>, AuthError> {
        let metrics = telemetry::auth_metrics();
        metrics.login_attempts.add(1, &[]);

        info!("Attempting password authentication");

        let user_result = user::Entity::find_by_email(&self.pool, email.clone()).await;

        let user = match user_result {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("User not found");
                tracing::Span::current().record("result", "user_not_found");
                metrics.login_failure.add(
                    1,
                    &[opentelemetry::KeyValue::new("reason", "user_not_found")],
                );
                return Ok(None);
            }
            Err(err) => {
                error!(error = ?err, "Database error during user lookup");
                metrics
                    .login_failure
                    .add(1, &[opentelemetry::KeyValue::new("reason", "db_error")]);
                return Err(AuthError::new(AuthErrorCode::BackendError)
                    .with_message("Database error during authentication"));
            }
        };

        // Check if user has a password (not OAuth user)
        let pwd_hash = match &user.password {
            Some(pwd) => pwd.clone(),
            None => {
                warn!("User has no password (OAuth user attempting password login)");
                tracing::Span::current().record("result", "no_password");
                metrics
                    .login_failure
                    .add(1, &[opentelemetry::KeyValue::new("reason", "oauth_user")]);
                return Ok(None);
            }
        };

        // Verify password in blocking task
        let verify_start = Instant::now();
        let password_valid =
            match task::spawn_blocking(move || Self::check_password(password, &pwd_hash)).await {
                Ok(result) => result?,
                Err(join_err) => {
                    error!(error = %join_err, "Password verification task failed");
                    metrics
                        .login_failure
                        .add(1, &[opentelemetry::KeyValue::new("reason", "task_error")]);
                    return Err(AuthError::new(AuthErrorCode::InternalError)
                        .with_message("Password verification failed"));
                }
            };

        let verify_duration = verify_start.elapsed().as_millis() as f64;
        metrics
            .password_verification_duration
            .record(verify_duration, &[]);

        if password_valid {
            info!(user_id = user.id, "Authentication successful");
            tracing::Span::current().record("result", "success");
            metrics.login_success.add(1, &[]);
            metrics.session_created.add(1, &[]);
            Ok(Some(user))
        } else {
            warn!("Invalid password");
            tracing::Span::current().record("result", "invalid_password");
            metrics.login_failure.add(
                1,
                &[opentelemetry::KeyValue::new("reason", "invalid_password")],
            );
            Ok(None)
        }
    }

    /// Authenticate with OAuth (Google ID)
    #[instrument(skip(self), fields(result))]
    pub async fn authenticate_oauth(
        &self,
        google_id: String,
    ) -> Result<Option<user::Model>, AuthError> {
        let metrics = telemetry::auth_metrics();
        info!("OAuth authentication attempt");

        let user = user::Entity::find_by_google_id(&self.pool, google_id)
            .await
            .map_err(|err| {
                error!(error = ?err, "Database error during OAuth user lookup");
                AuthError::new(AuthErrorCode::BackendError)
                    .with_message("Database error during OAuth lookup")
            })?;

        match user {
            Some(user) => {
                info!(user_id = user.id, "OAuth authentication successful");
                metrics.login_success.add(1, &[]);
                metrics.session_created.add(1, &[]);
                Ok(Some(user))
            }
            None => {
                warn!("OAuth user not found");
                metrics.login_failure.add(
                    1,
                    &[opentelemetry::KeyValue::new(
                        "reason",
                        "oauth_user_not_found",
                    )],
                );
                Ok(None)
            }
        }
    }
}

impl std::fmt::Debug for AuthBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthBackend")
            .field("pool", &"Pool{...}")
            .finish()
    }
}

/// Implement rux-auth's AuthUser trait for user::Model
impl AuthUser for user::Model {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // For OAuth users without passwords, use email as session hash
        match &self.password {
            Some(password) => password.as_bytes(),
            None => self.email.as_bytes(),
        }
    }

    fn email_verified(&self) -> bool {
        self.is_verified
    }

    fn totp_enabled(&self) -> bool {
        self.two_fa_enabled
    }

    fn role_level(&self) -> i32 {
        self.role.to_i32()
    }
}

/// Implement rux-auth's AuthBackend trait
#[async_trait]
impl RuxAuthBackend for AuthBackend {
    type User = user::Model;

    #[instrument(skip(self), fields(user_id = %id))]
    async fn get_user(&self, id: &i32) -> Result<Option<Self::User>, AuthError> {
        user::Entity::get_by_id(&self.pool, *id)
            .await
            .map_err(|err| {
                error!(error = ?err, "Error retrieving user");
                AuthError::new(AuthErrorCode::BackendError).with_message("Failed to retrieve user")
            })
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn check_ban(&self, user_id: &i32) -> Result<BanStatus, AuthError> {
        let ban = user_ban::Entity::get_active_ban(&self.pool, *user_id)
            .await
            .map_err(|err| {
                error!(error = ?err, "Error checking ban status");
                AuthError::new(AuthErrorCode::BackendError)
                    .with_message("Failed to check ban status")
            })?;

        match ban {
            Some(ban) => Ok(BanStatus::Banned {
                reason: ban.reason,
                expires_at: ban.expires_at,
                banned_by: ban.banned_by.map(|id| id as i64),
            }),
            None => Ok(BanStatus::NotBanned),
        }
    }

    #[instrument(skip(self, password), fields(user_id = %user_id))]
    async fn verify_password(&self, user_id: &i32, password: &str) -> Result<bool, AuthError> {
        let user = self.get_user(user_id).await?;

        let user = match user {
            Some(u) => u,
            None => return Ok(false),
        };

        let pwd_hash = match &user.password {
            Some(pwd) => pwd.clone(),
            None => return Ok(false), // OAuth user
        };

        let password = password.to_string();
        match task::spawn_blocking(move || Self::check_password(password, &pwd_hash)).await {
            Ok(result) => result.map_err(|_| AuthError::new(AuthErrorCode::InvalidCredentials)),
            Err(_) => Err(AuthError::new(AuthErrorCode::InternalError)
                .with_message("Password verification task failed")),
        }
    }

    async fn on_login(&self, user: &Self::User) -> Result<(), AuthError> {
        info!(user_id = user.id, "User logged in via rux-auth");
        Ok(())
    }

    async fn on_logout(&self, user_id: &i32) -> Result<(), AuthError> {
        info!(user_id = user_id, "User logged out via rux-auth");
        Ok(())
    }
}

/// Session revocation set key in Redis.
///
/// Members are tower-session ids (`Id::Display`, 22-char base64url). TTL is
/// refreshed to [`SESSION_MAX_AGE_SECS`] on each insert so the set self-cleans
/// as sessions age out of the store entirely.
pub fn revoked_set_key() -> String {
    "rux:revoked_sessions".to_string()
}

/// Mirror of the `SessionManagerLayer` inactivity expiry (14 days, in seconds).
/// Used to TTL the revocation set so it cannot grow without bound.
pub const SESSION_MAX_AGE_SECS: i64 = 14 * 24 * 60 * 60;

/// V-HIGH-2 revocation enforcement.
///
/// Revocation is enforced by [`AuthBackend::delete_tower_session`], which the
/// `sessions_terminate` handler runs (it has `state.redis_pool`): it `DEL`s the
/// live tower-sessions Redis key so the next request with that cookie finds no
/// session and is anonymous, and `SADD`s the id to a revocation set as
/// defense-in-depth.
///
/// This trait method is the per-request hook the extractor consults, but
/// `AuthBackend` does NOT hold a Redis pool (its `FromRef`/middleware
/// construction only provides the DB), so it returns `Ok(false)` today — i.e.
/// there is no per-request revocation check. This is a deliberate best-effort
/// tradeoff (mirrors the rate limiter's fail-open policy): revocation works as
/// long as the `DEL` at terminate-time succeeds; if Redis is unavailable at
/// that instant, the `revoked_at` stamp is audit/UI-only and does NOT enforce
/// — the cookie stays valid until its 14-day inactivity expiry. A hard
/// per-request guarantee would require plumbing a Redis pool into
/// `AuthBackend` (state.rs `FromRef` + the `auth_guard` middleware) and
/// overriding this to `SISMEMBER revoked_set_key()`.
#[async_trait]
impl SessionRevocation for AuthBackend {
    async fn is_session_revoked(&self, _tower_session_id: &str) -> Result<bool, AuthError> {
        Ok(false)
    }
}
