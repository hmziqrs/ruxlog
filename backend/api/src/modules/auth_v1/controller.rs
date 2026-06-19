use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;

use axum_client_ip::ClientIp;

use rux_auth::AuthBackend as AuthBackendTrait;
use sea_orm::ActiveModelTrait;
use serde_json::json;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{email_verification, user, user_session},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    modules::auth_v1::validator::{
        V1LoginPayload, V1RegisterPayload, V1TwoFADisablePayload, V1TwoFAVerifyPayload,
    },
    services::{abuse_limiter, auth::AuthSession, mail::send_email_verification_code},
    utils::twofa,
    AppState,
};

/// Two-tier, fail-closed limiter config reused for brute-force-sensitive
/// auth endpoints (here: the 2FA code verify). Mirrors the config used by the
/// forgot-password and email-verification flows.
const ABUSE_LIMITER_CONFIG: abuse_limiter::AbuseLimiterConfig = abuse_limiter::AbuseLimiterConfig {
    temp_block_attempts: 3,
    temp_block_range: 360,
    temp_block_duration: 3600,
    block_retry_limit: 5,
    block_range: 900,
    block_duration: 86400,
};

#[debug_handler(state = AppState)]
#[instrument(skip(auth), fields(user_id))]
pub async fn log_out(mut auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    if let Some(user) = &auth.user {
        tracing::Span::current().record("user_id", user.id);
        info!(user_id = user.id, "User logging out");
    }

    match auth.logout().await {
        Ok(_) => {
            info!("Logout successful");
            Ok((StatusCode::OK, Json(json!({"message": "Logged out"}))))
        }
        Err(e) => {
            error!(error = %e, "Logout failed");
            Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("An error occurred while logging out"))
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth, payload), fields(client_ip = %secure_ip, user_id, user_role, result))]
pub async fn log_in(
    State(state): State<AppState>,
    mut auth: AuthSession,
    ClientIp(secure_ip): ClientIp,
    headers: HeaderMap,
    payload: ValidatedJson<V1LoginPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!(client_ip = %secure_ip, "Login attempt");

    let payload = payload.0;
    let user = auth
        .backend()
        .authenticate_password(payload.email, payload.password)
        .await;

    match user {
        Ok(Some(user)) => {
            tracing::Span::current().record("user_id", user.id);
            tracing::Span::current().record("user_role", user.role.to_string());

            // Reject banned users at the door — never issue a session for a
            // banned account. Fail closed if the ban lookup itself errors so a
            // transient DB/Redis failure cannot grant access to a banned user.
            match auth.backend().check_ban(&user.id).await {
                Ok(ban_status) if ban_status.is_banned() => {
                    warn!(user_id = user.id, "Banned user attempted login");
                    tracing::Span::current().record("result", "banned");
                    return Err(ErrorResponse::new(ErrorCode::AccountLocked)
                        .with_message("This account has been banned"));
                }
                Ok(_) => {}
                Err(_) => {
                    tracing::Span::current().record("result", "ban_check_error");
                    return Err(ErrorResponse::new(ErrorCode::InternalServerError)
                        .with_message("Unable to verify account status"));
                }
            }

            // NOTE (audit F#4 — intentionally deferred): a correct password is
            // sufficient to obtain a fully authenticated session here, EVEN for
            // users with 2FA enrolled. The login flow does not issue a partial
            // session that demands a TOTP challenge, and no protected route
            // enforces `totp_if_enabled()` (see rux-auth `check_requirements`).
            // 2FA is therefore self-serve / opt-in-for-UI only at the access
            // boundary. This is the locked "leave the login flow as-is, fix
            // leaks only" decision — the TOTP-seed leak and backup-code bias
            // HAVE been fixed; enforcing 2FA at login is explicitly out of
            // scope. Reversing it is a one-line `.totp_if_enabled()` chain on
            // the live guards plus a two-step login response. See
            // `docs/CRYPTO_AUDIT.md` → "Accepted Deferrals".

            let ip = Some(secure_ip.to_string());
            let device = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match auth
                .login_with_metadata(&user, device.clone(), ip.clone())
                .await
            {
                Ok(_) => {
                    info!(
                        user_id = user.id,
                        user_role = user.role.to_string(),
                        device = ?device,
                        "Login successful"
                    );

                    let session_row = user_session::Entity::create(
                        &state.sea_db,
                        user_session::NewUserSession::new(user.id, device, ip),
                    )
                    .await
                    .ok();

                    // V-HIGH-2: persist the PG-row → tower-session-id mapping in
                    // Redis so `sessions_terminate` can later find and DEL the
                    // live tower-sessions record. Without this, terminating a
                    // session only stamps `revoked_at` and the cookie keeps
                    // authenticating for up to 14 days. The mapping TTLs out
                    // with the session max-age so it self-cleans.
                    //
                    // `login_with_metadata` calls `cycle_id`, which sets the
                    // in-memory session id to `None` until the record is saved.
                    // Save now to materialize the cycled id (the one the cookie
                    // will carry) before reading it. The `SessionManagerLayer`
                    // saves again at response time — that just updates the same
                    // record (`store.save` path), so this is safe and
                    // idempotent.
                    if (auth.session().save().await).is_ok() {
                        if let (Some(row), Some(tower_sid)) =
                            (session_row.as_ref(), auth.session().id())
                        {
                            record_session_mapping(
                                &state.redis_pool,
                                row.id,
                                &tower_sid.to_string(),
                            )
                            .await;
                        }
                    }

                    tracing::Span::current().record("result", "success");
                    Ok((StatusCode::OK, Json(json!(user))))
                }
                Err(err) => {
                    error!(error = %err, user_id = user.id, "Session creation failed");
                    tracing::Span::current().record("result", "session_error");
                    Err(ErrorResponse::new(ErrorCode::InternalServerError)
                        .with_message("An error occurred while logging in")
                        .with_details(err.to_string()))
                }
            }
        }
        Ok(None) => {
            warn!(client_ip = %secure_ip, "Invalid credentials");
            tracing::Span::current().record("result", "invalid_credentials");
            Err(ErrorResponse::new(ErrorCode::InvalidCredentials))
        }
        Err(err) => {
            error!(error = ?err, client_ip = %secure_ip, "Authentication error");
            tracing::Span::current().record("result", "auth_error");
            Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Authentication error"))
        }
    }
}

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(state, payload), fields(user_id, result))]
pub async fn register(
    State(state): State<AppState>,
    payload: ValidatedJson<V1RegisterPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0;

    info!("User registration attempt");

    let email = payload.email.clone();

    // Generate the verification code now: store only its keyed hash alongside
    // the new user (transactional), but email the plaintext. The plaintext
    // never touches the database (audit: "brute-forceable plaintext codes" —
    // fixed in Phase 3d).
    let code = email_verification::Entity::generate_code();
    let code_hash = crate::utils::code_hash::hash_code(&state.secret_key, &code);

    match user::Entity::create(&state.sea_db, payload.into_new_user(), code_hash).await {
        Ok(user) => {
            info!(user_id = user.id, email = %user.email, "User registered successfully");
            tracing::Span::current().record("user_id", user.id);
            tracing::Span::current().record("result", "success");

            // Send first-party email verification code (non-blocking)
            let app_state = state.clone();
            let user_id = user.id;
            let email_for_task = email.clone();
            let code_for_task = code.clone();
            tokio::spawn(async move {
                if let Err(err) =
                    send_email_verification_code(&app_state.mailer, &email_for_task, &code_for_task)
                        .await
                {
                    tracing::error!(
                        user_id,
                        email = %email_for_task,
                        "Failed to send verification email: {}",
                        err
                    );
                }
            });

            Ok((StatusCode::CREATED, Json(json!(user))))
        }
        Err(err) => {
            warn!(error = ?err, "Registration failed");
            tracing::Span::current().record("result", "failure");
            Err(err)
        }
    }
}

#[cfg(feature = "auth-2fa")]
#[debug_handler]
#[instrument(skip(state, auth), fields(user_id))]
pub async fn twofa_setup(
    State(state): State<AppState>,
    auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    tracing::Span::current().record("user_id", user.id);

    info!(user_id = user.id, "2FA setup initiated");

    // Generate base32 secret and backup codes, persist to user.
    // CSPRNG failures must surface as 500s — never silently produce a
    // predictable secret. See plan Phase 2f.
    let secret_b32 = twofa::generate_secret_base32(20)
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to generate 2FA secret")
        })?;
    let otpauth_url = twofa::build_otpauth_url(
        &user.email,
        "Ruxlog",
        &secret_b32,
        twofa::DEFAULT_TOTP_DIGITS,
    );

    // Generate and hash backup codes (store Argon2id hashes only).
    let backup_codes = twofa::generate_backup_codes(10)
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to generate backup codes")
        })?;
    // Argon2id is memory-hard; hash off the async worker thread.
    let backup_hashes = {
        let codes_for_hash = backup_codes.clone();
        tokio::task::spawn_blocking(move || twofa::hash_backup_codes(&codes_for_hash))
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message(format!("Backup code hashing failed: {e}"))
            })?
    };
    let backup_hashes_json = serde_json::json!(backup_hashes);

    // Persist on user
    let existing = user::Entity::find_by_id_with_404(&state.sea_db, user.id).await?;
    let mut active: user::ActiveModel = existing.into();
    active.two_fa_enabled = sea_orm::Set(false);
    active.two_fa_secret = sea_orm::Set(Some(secret_b32.clone()));
    active.two_fa_backup_codes = sea_orm::Set(Some(backup_hashes_json));
    active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
    active.update(&state.sea_db).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "secret": secret_b32,
            "otpauth_url": otpauth_url,
            "backup_codes": backup_codes,
        })),
    ))
}

#[cfg(feature = "auth-2fa")]
#[debug_handler]
pub async fn twofa_verify(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1TwoFAVerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let payload = payload.0;

    // Throttle brute-force attempts on the 2FA code. Fail-closed: a Redis
    // outage denies the attempt rather than letting unbounded tries through.
    let key_prefix = format!("totp:{}", user.id);
    abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await?;

    let existing = user::Entity::find_by_id_with_404(&state.sea_db, user.id).await?;
    let secret = match &existing.two_fa_secret {
        Some(s) => s.clone(),
        None => {
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("2FA not initialized"))
        }
    };

    // If code matches TOTP, enable 2FA. Otherwise, try backup code consumption.
    let totp_ok = twofa::verify_totp_code_now(&secret, &payload.code);

    if totp_ok {
        let mut active: user::ActiveModel = existing.into();
        active.two_fa_enabled = sea_orm::Set(true);
        active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
        let updated = active.update(&state.sea_db).await?;
        return Ok((StatusCode::OK, Json(json!(updated))));
    }

    // Try backup code if provided
    if let Some(backup_code) = payload.backup_code {
        if let Some(stored) = &existing.two_fa_backup_codes {
            // Materialize owned copies and end the borrow on `existing` before
            // awaiting: Argon2id verification runs on the blocking pool.
            let stored_vec: Vec<String> =
                serde_json::from_value(stored.clone()).unwrap_or_else(|_| vec![]);
            let consume_result = {
                let stored_clone = stored_vec;
                let code_clone = backup_code.clone();
                tokio::task::spawn_blocking(move || {
                    twofa::consume_backup_code(&stored_clone, &code_clone)
                })
                .await
                .map_err(|e| {
                    ErrorResponse::new(ErrorCode::InternalServerError)
                        .with_message(format!("Backup code verification failed: {e}"))
                })?
            };
            if let Some(updated_hashes) = consume_result {
                let mut active: user::ActiveModel = existing.into();
                active.two_fa_enabled = sea_orm::Set(true);
                active.two_fa_backup_codes = sea_orm::Set(Some(serde_json::json!(updated_hashes)));
                active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
                let updated = active.update(&state.sea_db).await?;
                return Ok((StatusCode::OK, Json(json!(updated))));
            }
        }
    }

    Err(ErrorResponse::new(ErrorCode::InvalidToken).with_message("Invalid 2FA code"))
}

#[cfg(feature = "auth-2fa")]
#[debug_handler]
pub async fn twofa_disable(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1TwoFADisablePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let payload = payload.0;

    // Throttle brute-force attempts on the 2FA code (audit V-MED-4): without
    // this, an attacker holding the password could mount unbounded online TOTP
    // guessing against `twofa_disable` to turn off a victim's 2FA. Shares the
    // `totp:{user.id}` budget with `twofa_verify` so attempts across both
    // endpoints count against one limit. Fail-closed on Redis outage.
    let key_prefix = format!("totp:{}", user.id);
    abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await?;

    let existing = user::Entity::find_by_id_with_404(&state.sea_db, user.id).await?;

    // If 2FA is enabled and a code is provided, verify it; allow disable with valid code or backup code
    if existing.two_fa_enabled {
        if let Some(code) = payload.code.clone() {
            let secret = existing.two_fa_secret.clone().unwrap_or_default();
            let totp_ok = if secret.is_empty() {
                false
            } else {
                twofa::verify_totp_code_now(&secret, &code)
            };

            let mut backup_ok = false;
            if !totp_ok {
                if let Some(stored) = &existing.two_fa_backup_codes {
                    let stored_vec: Vec<String> =
                        serde_json::from_value(stored.clone()).unwrap_or_else(|_| vec![]);
                    // Argon2id verification is memory-hard; run off the async
                    // worker. The borrow of `existing` via `stored` ends here
                    // (only `stored_vec`, owned, crosses the await).
                    backup_ok = {
                        let stored_clone = stored_vec;
                        let code_clone = code.clone();
                        tokio::task::spawn_blocking(move || {
                            twofa::consume_backup_code(&stored_clone, &code_clone).is_some()
                        })
                        .await
                        .map_err(|e| {
                            ErrorResponse::new(ErrorCode::InternalServerError)
                                .with_message(format!("Backup code verification failed: {e}"))
                        })?
                    };
                }
            }

            if !totp_ok && !backup_ok {
                return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                    .with_message("Invalid 2FA or backup code"));
            }
        } else {
            // Require a code if 2FA is enabled
            return Err(ErrorResponse::new(ErrorCode::MissingRequiredField)
                .with_message("code is required"));
        }
    }

    // Disable and clear secrets
    let mut active: user::ActiveModel = existing.into();
    active.two_fa_enabled = sea_orm::Set(false);
    active.two_fa_secret = sea_orm::Set(None);
    active.two_fa_backup_codes = sea_orm::Set(None);
    active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
    let updated = active.update(&state.sea_db).await?;

    Ok((StatusCode::OK, Json(json!(updated))))
}

#[debug_handler]
pub async fn sessions_list(
    State(state): State<AppState>,
    auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let page = 1;

    match user_session::Entity::list_by_user(&state.sea_db, user.id, Some(page)).await {
        Ok((sessions, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": sessions,
                "total": total,
                "page": page,
            })),
        )),
        Err(err) => Err(err),
    }
}

#[debug_handler]
pub async fn sessions_terminate(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user_id = auth.user.as_ref().map(|u| u.id).unwrap_or(0);
    let result = user_session::Entity::revoke(&state.sea_db, id).await?;
    match result {
        Some(session) if session.user_id == user_id => {
            // V-HIGH-2: actually invalidate the live tower-sessions record.
            // `Entity::revoke` only stamps `user_sessions.revoked_at` (audit/
            // UI-only — nothing on the auth path reads it); the tower-sessions
            // Redis key remains, so the cookie keeps authenticating. Look up the
            // tower-session id captured at login and DEL it from Redis (plus
            // add it to a revocation set as defense-in-depth). A missing mapping
            // (a session whose login path didn't record one, or a pre-fix row)
            // means we CANNOT DEL the live record — the cookie then stays valid
            // until its 14-day inactivity expiry. `revoked_at` does NOT enforce
            // that window; it is audit-only.
            if let Some(tower_sid) = lookup_session_mapping(&state.redis_pool, id).await {
                auth.backend()
                    .delete_tower_session(&state.redis_pool, &tower_sid)
                    .await;
            } else {
                warn!(
                    session_id = id,
                    "No tower-session mapping for session; live record could not be DEL'd — cookie valid until 14-day expiry (revoked_at is audit-only)"
                );
            }
            Ok((StatusCode::OK, Json(json!({ "message": "Session terminated" }))))
        }
        Some(_) => Err(ErrorResponse::new(ErrorCode::Unauthorized)),
        None => Err(ErrorResponse::new(ErrorCode::RecordNotFound)),
    }
}

/// Redis key holding the tower-session id for a given `user_sessions.id`.
fn session_mapping_key(pg_session_id: i32) -> String {
    format!("rux:sid_map:{pg_session_id}")
}

/// Persist `user_sessions.id -> tower_session_id` so terminate can later find
/// and kill the live tower-sessions record. TTLs with the session max-age.
/// `pub(crate)` so the Google-OAuth login path records the same mapping.
pub(crate) async fn record_session_mapping(
    redis_pool: &tower_sessions_redis_store::fred::prelude::Pool,
    pg_session_id: i32,
    tower_session_id: &str,
) {
    use tower_sessions_redis_store::fred::interfaces::KeysInterface;

    let key = session_mapping_key(pg_session_id);
    let set_result: Result<(), _> = redis_pool
        .set::<(), _, _>(
            key,
            tower_session_id.to_string(),
            Some(fred::types::Expiration::EX(
                crate::services::auth::SESSION_MAX_AGE_SECS,
            )),
            None,
            false,
        )
        .await;
    if let Err(e) = set_result {
        warn!(error = %e, "Failed to record tower-session mapping");
    }
}

/// Look up the tower-session id previously recorded for a `user_sessions.id`.
/// Returns `None` if the mapping is absent (pre-fix rows, or expired).
async fn lookup_session_mapping(
    redis_pool: &tower_sessions_redis_store::fred::prelude::Pool,
    pg_session_id: i32,
) -> Option<String> {
    use tower_sessions_redis_store::fred::interfaces::KeysInterface;

    let key = session_mapping_key(pg_session_id);
    match redis_pool.get::<Option<String>, _>(key).await {
        Ok(Some(sid)) => Some(sid),
        Ok(None) => None,
        Err(e) => {
            warn!(
                error = %e,
                session_id = pg_session_id,
                "Failed to look up tower-session mapping; live record cannot be DEL'd (revoked_at is audit-only)"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::session_mapping_key;

    /// V-HIGH-2: the PG-row → tower-session-id mapping key must be a stable,
    /// namespaced function of the integer `user_sessions.id` so terminate can
    /// recover the exact key login wrote.
    #[test]
    fn session_mapping_key_is_stable_and_namespaced() {
        assert_eq!(session_mapping_key(1), "rux:sid_map:1");
        assert_eq!(session_mapping_key(42), "rux:sid_map:42");
        assert_eq!(
            session_mapping_key(7),
            format!("rux:sid_map:{}", 7),
            "key must be the pg id under the rux namespace"
        );
    }
}
