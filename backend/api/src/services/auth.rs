use async_trait::async_trait;
use password_auth::verify_password;
use rux_auth::{
    AuthBackend as RuxAuthBackend, AuthError, AuthErrorCode, AuthUser, BanStatus, SessionRevocation,
};
use sea_orm::DatabaseConnection;
use std::sync::LazyLock;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, instrument, warn};

use crate::{db::sea_models::user, db::sea_models::user_ban, utils::telemetry};

/// CRYP-SC-002 (login timing equalization): a fixed, non-secret Argon2id hash
/// used to drive a DUMMY password verify on the user-not-found and
/// OAuth-no-password branches of [`AuthBackend::authenticate_password`].
///
/// The valid-user / wrong-password branch performs a full Argon2id
/// `verify_password` that dominates the request's CPU cost; the two early-return
/// branches did none of that work, so an attacker averaging request latency
/// could distinguish "valid user, wrong password" from "no such user" / "OAuth
/// user". To close that timing oracle we run the SAME verify against THIS fixed
/// hash before returning, so every login attempt pays the Argon2id cost
/// regardless of branch. The input password is verified against a hash it can
/// never match, so the result is always `false` — exactly the cost shape of a
/// genuine wrong-password attempt.
///
/// This mirrors the existing `equalize_unknown_email_work` pattern from
/// `modules/forgot_password_v1/controller.rs`. We compute the hash ONCE
/// (process lifetime) via `LazyLock` rather than hard-coding a PHC string, so
/// the Argon2 parameters always track the `password_auth` crate version used by
/// the rest of the codebase (no drift between the dummy hash and real hashes).
const DUMMY_VERIFY_PASSWORD: &str = "timing-equalization-dummy-fixture";
static DUMMY_VERIFY_HASH: LazyLock<String> =
    LazyLock::new(|| password_auth::generate_hash(DUMMY_VERIFY_PASSWORD));

/// Re-export the AuthSession from rux-auth
pub type AuthSession = rux_auth::AuthSession<AuthBackend>;

/// Authentication backend implementation.
///
/// Holds BOTH the DB pool and the Redis pool. The Redis pool is required for
/// the per-request session-revocation check (V-HIGH-2): on every authenticated
/// request the extractor consults [`SessionRevocation::is_session_revoked`],
/// which does a Redis `SISMEMBER` against [`revoked_set_key`] to see whether
/// the live tower-session id was administratively revoked. Previously
/// `AuthBackend` held only the DB pool, so `is_session_revoked` was a hard-coded
/// `Ok(false)` and revocation worked only if the one-shot `DEL` at terminate
/// time won the race against a concurrent session save.
#[derive(Clone)]
pub struct AuthBackend {
    pub pool: DatabaseConnection,
    /// Same pool type as `AppState::redis_pool` / `delete_tower_session`.
    pub redis_pool: tower_sessions_redis_store::fred::prelude::Pool,
}

impl AuthBackend {
    pub fn new(
        pool: &DatabaseConnection,
        redis_pool: tower_sessions_redis_store::fred::prelude::Pool,
    ) -> Self {
        Self {
            pool: pool.clone(),
            redis_pool,
        }
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
        let sadd_result: Result<i64, _> = redis_pool
            .sadd(revoked_key.clone(), vec![tower_session_id.to_string()])
            .await;
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

    /// CRYP-SC-002: run a DUMMY Argon2id verify of `password` against the fixed
    /// [`DUMMY_VERIFY_HASH`]. The result is always false (the password can never
    /// match the fixture hash) — the work is what matters, not the outcome.
    /// Returns an error only on a genuine hashing failure, matching
    /// [`check_password`]'s contract so the caller's error handling is uniform.
    fn run_dummy_password_verify(password: String) -> Result<bool, AuthError> {
        // `LazyLock` is initialized on first touch; safe to read from a blocking
        // task. `as_str()` borrows the static backing storage, so the clone-free
        // borrow is valid for the lifetime of the process.
        let hash: &str = DUMMY_VERIFY_HASH.as_str();
        verify_password(password, hash)
            .map(|_| true)
            .map_err(|_| AuthError::new(AuthErrorCode::InvalidCredentials))
    }

    /// CRYP-SC-002: run [`run_dummy_password_verify`] off the async executor, so
    /// the memory-hard Argon2id KDF never blocks the Tokio runtime. Mirrors the
    /// wrong-password branch's `spawn_blocking` around `check_password`. The
    /// (always-false) result is dropped; only the CPU cost matters.
    async fn run_blocking_dummy_verify(password: String) -> Result<(), AuthError> {
        match task::spawn_blocking(move || Self::run_dummy_password_verify(password)).await {
            Ok(_) => Ok(()),
            Err(join_err) => {
                error!(error = %join_err, "Dummy password verification task failed");
                Err(AuthError::new(AuthErrorCode::InternalError)
                    .with_message("Password verification failed"))
            }
        }
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
                // CRYP-SC-002: equalize CPU with the valid-user / wrong-password
                // branch, which pays a full Argon2id verify here. Without this
                // dummy verify the markedly cheaper not-found path is a timing
                // oracle for account existence. The result is always false (the
                // password can never match the fixture hash); only the work
                // matters. See [`DUMMY_VERIFY_HASH`].
                Self::run_blocking_dummy_verify(password).await?;
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
                // CRYP-SC-002: same timing equalization as the not-found branch
                // — an OAuth user (no password hash) would otherwise return far
                // faster than a wrong-password attempt, leaking "this account
                // exists but is OAuth-only". Run the dummy Argon2id verify.
                Self::run_blocking_dummy_verify(password).await?;
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
            .field("redis_pool", &"RedisPool{...}")
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
        // CRYP-ENC-004: the per-session CSRF/session-auth binding is keyed on the
        // server-random `session_auth_secret` column (added by the model layer /
        // W3) rather than the raw `email`. Basing it on email meant an email
        // change OR a sufficiently strong attacker who could observe the derived
        // hash could recompute it from a public field; a per-user random secret
        // is not derivable from any user-facing value. Rotating the secret (e.g.
        // on credential change) invalidates prior sessions, which is the desired
        // trust-transition behavior.
        //
        // Defense-in-depth fallback: if the secret column is unexpectedly absent
        // (should not occur after the backfill migration writes a random secret
        // per existing user), fall back to the password hash for password users
        // rather than panic — this keeps the session functional while still
        // avoiding the raw-email path. OAuth users without a secret have nothing
        // stable to fall back to, so an empty hash forces session invalidation.
        if !self.session_auth_secret.is_empty() {
            return self.session_auth_secret.as_bytes();
        }
        match &self.password {
            Some(password) => password.as_bytes(),
            None => &[],
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

/// V-HIGH-2 real per-request session revocation.
///
/// `AuthBackend::delete_tower_session` (run by the `sessions_terminate` handler)
/// both `DEL`s the live tower-sessions key AND `SADD`s the id into
/// [`revoked_set_key`] as defense-in-depth. This trait method is the
/// per-request hook the extractor consults on every authenticated request: it
/// runs a Redis `SISMEMBER` against [`revoked_set_key`] for the live
/// tower-session id. If the id is a member, the extractor deletes the session
/// and treats the caller as unauthenticated — so a revoked cookie stops
/// authenticating on the *very next request* even if the terminate-time `DEL`
/// raced a concurrent session save.
///
/// **Cost:** one extra Redis round-trip (`SISMEMBER`, O(1)) per authenticated
/// request. This is acceptable and standard — `tower-sessions` already performs
/// a per-request Redis `GET` to load the session, so this adds one more
/// constant-time lookup.
///
/// **Fail-open policy (unchanged from the prior no-op):** on a Redis error we
/// return `Ok(false)` and `warn!`. This mirrors the rate limiter's fail-open
/// behavior and avoids a mass lockout during a transient Redis blip — at the
/// cost of a revoked session briefly staying live until Redis recovers. The
/// DB `revoked_at` stamp and the session's own 14-day inactivity expiry still
/// bound the window.
#[async_trait]
impl SessionRevocation for AuthBackend {
    #[instrument(skip(self), level = "debug")]
    async fn is_session_revoked(&self, tower_session_id: &str) -> Result<bool, AuthError> {
        use tower_sessions_redis_store::fred::interfaces::SetsInterface;

        let key = revoked_set_key();
        match self
            .redis_pool
            .sismember::<bool, _, _>(key, tower_session_id)
            .await
        {
            Ok(is_member) => Ok(is_member),
            Err(e) => {
                // Fail-open: a Redis blip must not lock out every user. The
                // `revoked_at` audit row, the terminate-time `DEL`, and the
                // 14-day inactivity expiry still bound the revocation window.
                warn!(
                    error = %e,
                    tower_session_id = %tower_session_id,
                    "Revocation SISMEMBER failed (fail-open): session remains valid until Redis recovers"
                );
                Ok(false)
            }
        }
    }
}
