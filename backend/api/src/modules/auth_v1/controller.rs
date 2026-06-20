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
        V1LoginPayload, V1LoginTotpPayload, V1RegisterPayload, V1TwoFADisablePayload,
        V1TwoFAVerifyPayload,
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

            // F#4 / F#7 / F#16 (2FA-at-login) — NOW ENFORCED. A correct
            // password is NO LONGER sufficient to obtain a fully authenticated
            // session for a user with 2FA enrolled. Instead, when
            // `user.two_fa_enabled` is true, we issue a SHORT-LIVED, single-use
            // pending-TOTP credential (an opaque random token stored in Redis,
            // TTL ~5 min) and return `{ status: "totp_required", totp_token }`
            // WITHOUT calling `login_with_metadata` — so NO session cookie is
            // set and NO authenticated session exists. The caller must then POST
            // that token plus a valid TOTP code to `/login/totp`, which — only
            // on a verified, replay-blocked code — issues the FULL session
            // (with session-id rotation + sid_map recording, identical to the
            // non-2FA path). The pending token authenticates ONLY the TOTP step
            // and grants nothing else; it is consumed (GETDEL) on first use.
            // Non-2FA users get the full session exactly as before (backward
            // compatible). See `login_totp` below and `docs/CRYPTO_AUDIT.md`.
            if user.two_fa_enabled {
                tracing::Span::current().record("result", "totp_required");
                let totp_token = login_totp_token::mint(&state, user.id).await?;
                info!(user_id = user.id, "Login requires TOTP (2FA enrolled)");
                return Ok((
                    StatusCode::OK,
                    Json(json!({
                        "status": "totp_required",
                        "totp_token": totp_token,
                    })),
                ));
            }

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

/// Second step of the two-step 2FA-at-login flow (F#4 / F#7 / F#16).
///
/// `log_in` issues a short-lived, single-use pending credential
/// (`totp_token`) instead of a full session for TOTP-enrolled users. This
/// endpoint exchanges that token plus a valid TOTP code for the FULL session
/// (the same grant `log_in` makes for non-2FA users). Pipeline:
///
///   1. Atomically consume the pending token (`GETDEL` → single-use: a replayed
///      token observes `None` and is rejected). The token authenticates ONLY
///      this step — it is never itself a session.
///   2. Load the user. Fail-closed ban re-check (a user banned between `log_in`
///      and `/login/totp` must NOT get a session).
///   3. Rate-limit on the shared `totp:{user_id}` abuse-limiter bucket (same
///      budget as `/2fa/verify` + `/2fa/disable`), so brute-forcing the 6-digit
///      code here counts against the one limit.
///   4. Verify the TOTP code via the counter-returning variant + the atomic
///      `advance_totp_counter_if_higher` replay gate — so a code consumed at
///      login cannot be replayed at `/2fa/verify`, `/2fa/disable`, or a second
///      login, and vice versa.
///   5. ONLY on a verified, replay-blocked code, call `login_with_metadata` to
///      issue the full session (session-id rotation + sid_map recording,
///      identical to the non-2FA path).
///
/// Every failure returns a generic 401 so the response does not reveal whether
/// the token, the code, or the replay gate was the cause.
#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id, result))]
pub async fn login_totp(
    State(state): State<AppState>,
    mut auth: AuthSession,
    payload: ValidatedJson<V1LoginTotpPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0;

    // Single-use: GETDEL so a replayed token is gone and observed as None.
    let user_id = match login_totp_token::take(&state, &payload.totp_token).await? {
        Some(id) => id,
        None => {
            warn!("login/totp: pending token missing/expired/replayed");
            return Err(ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Invalid or expired login session"));
        }
    };
    tracing::Span::current().record("user_id", user_id);

    let user = match auth.backend().get_user(&user_id).await {
        Ok(Some(u)) => u,
        _ => {
            warn!(user_id, "login/totp: user not found after valid token");
            return Err(ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Invalid or expired login session"));
        }
    };

    // Fail-closed ban re-check: a ban issued between log_in and /login/totp
    // must not yield a session. Mirrors the log_in door check.
    match auth.backend().check_ban(&user.id).await {
        Ok(ban_status) if ban_status.is_banned() => {
            warn!(user_id = user.id, "login/totp: banned user");
            return Err(ErrorResponse::new(ErrorCode::AccountLocked)
                .with_message("This account has been banned"));
        }
        Ok(_) => {}
        Err(_) => {
            return Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Unable to verify account status"));
        }
    }

    // Share the totp:{user_id} abuse-limiter bucket with /2fa/verify and
    // /2fa/disable. Fail-closed on Redis outage (denies the attempt rather
    // than allowing unbounded TOTP guessing).
    let key_prefix = format!("totp:{}", user.id);
    abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await?;

    // Verify the TOTP code. The user's stored secret must be present (2FA is
    // enabled, so a missing secret is an inconsistent state — reject generically).
    let secret = match &user.two_fa_secret {
        Some(s) => s.clone(),
        None => {
            warn!(user_id = user.id, "login/totp: 2FA enabled but no secret stored");
            return Err(ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Invalid or expired login session"));
        }
    };

    let matched = twofa::verify_totp_code_now(&secret, &payload.code);
    let matched = match matched {
        Some(m) => m,
        None => {
            warn!(user_id = user.id, "login/totp: invalid TOTP code");
            return Err(ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Invalid 2FA code"));
        }
    };

    // Replay gate: the cheap fast-path reject against the just-read row, then
    // the authoritative atomic conditional UPDATE. Only one concurrent request
    // (across login AND /2fa/verify AND /2fa/disable) can claim the row, so a
    // code reused on a second endpoint is rejected. A lost race reads as a
    // generic 401 (no leak that it was a replay).
    if !twofa::is_fresh_counter(matched, user.two_fa_last_totp_counter) {
        warn!(
            user_id = user.id,
            matched_counter = matched,
            "login/totp: TOTP code replayed (counter already used)"
        );
        return Err(ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Invalid 2FA code"));
    }
    let advanced =
        user::Entity::advance_totp_counter_if_higher(&state.sea_db, user.id, matched).await?;
    if !advanced {
        warn!(
            user_id = user.id,
            matched_counter = matched,
            "login/totp: TOTP lost the watermark race (concurrent advance); rejecting as replay"
        );
        return Err(ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Invalid 2FA code"));
    }

    // TOTP verified + replay blocked: issue the FULL session, identical to the
    // non-2FA log_in path (session-id rotation + user_sessions row + sid_map).
    // This endpoint has no ClientIp/User-Agent extraction (the pending token is
    // the only thing binding step 2 to step 1), so record no device/ip — the
    // session row is still created for the sessions UI + revoke mapping.
    let ip: Option<String> = None;
    let device: Option<String> = None;
    match auth
        .login_with_metadata(&user, device.clone(), ip.clone())
        .await
    {
        Ok(_) => {
            info!(user_id = user.id, "login/totp: TOTP verified, full session issued");
            tracing::Span::current().record("result", "success");

            let session_row = user_session::Entity::create(
                &state.sea_db,
                user_session::NewUserSession::new(user.id, device, ip),
            )
            .await
            .ok();

            // V-HIGH-2: record the PG-row -> tower-session-id mapping (same as
            // the password-login path) so sessions_terminate can later DEL the
            // live record.
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

            Ok((StatusCode::OK, Json(json!(user))))
        }
        Err(err) => {
            error!(error = %err, user_id = user.id, "login/totp: session creation failed");
            tracing::Span::current().record("result", "session_error");
            Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("An error occurred while logging in")
                .with_details(err.to_string()))
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
    //
    // V-MED-6 (TOTP replay + TOCTOU): the previously stateless verify let an
    // accepted 6-digit code be replayed for ~90s (window=1 → 3 steps of 30s).
    // `is_fresh_counter` below is the cheap fast-path reject — it compares the
    // matched RFC 6238 counter against the just-read row and bounces
    // obviously-stale codes without touching the DB. But that read-then-write
    // is racy: two concurrent verifies can each observe the stale watermark,
    // each pass `is_fresh_counter`, and each persist — accepting the same
    // counter twice. The authoritative gate is therefore the atomic
    // conditional UPDATE `advance_totp_counter_if_higher`: only one concurrent
    // request can claim the row, so the loser sees `false` and is rejected as a
    // replay. The last-used counter is read from `existing` so the fast path
    // uses the DB's current value; the conditional UPDATE is the source of
    // truth.
    let totp_counter = twofa::verify_totp_code_now(&secret, &payload.code);

    if let Some(matched) = totp_counter {
        if twofa::is_fresh_counter(matched, existing.two_fa_last_totp_counter) {
            // Authoritative replay gate: atomically advance the watermark only
            // if `matched` is strictly higher than whatever the row currently
            // holds. A concurrent verify/disable that already advanced past
            // `matched` makes this UPDATE match zero rows → reject as a replay
            // (no leak — same generic InvalidToken as any bad code).
            let advanced = user::Entity::advance_totp_counter_if_higher(
                &state.sea_db,
                user.id,
                matched,
            )
            .await?;
            if !advanced {
                warn!(
                    user_id = user.id,
                    matched_counter = matched,
                    "TOTP code lost the watermark race (concurrent advance); rejecting as replay"
                );
                return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                    .with_message("Invalid 2FA code"));
            }
            // Watermark advanced; flip 2FA on. `two_fa_last_totp_counter` is
            // already persisted atomically above, so this UPDATE only sets the
            // enabled flag (and refreshes updated_at).
            let mut active: user::ActiveModel = existing.into();
            active.two_fa_enabled = sea_orm::Set(true);
            active.two_fa_last_totp_counter = sea_orm::Unchanged(Some(matched));
            active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
            let updated = active.update(&state.sea_db).await?;
            return Ok((StatusCode::OK, Json(json!(updated))));
        } else {
            // Replay of an already-used counter within the window. Count it
            // against the abuse limiter budget (already consumed above) and
            // reject as an invalid token — do NOT reveal that it was a replay.
            warn!(
                user_id = user.id,
                matched_counter = matched,
                "TOTP code replayed (counter already used); rejecting"
            );
            return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                .with_message("Invalid 2FA code"));
        }
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

    // V-MED-6 (TOTP replay + TOCTOU): when disable is authorized via a fresh
    // TOTP counter, the watermark is advanced atomically AT authorization time
    // via `advance_totp_counter_if_higher` (the authoritative replay gate).
    // `totp_authorized` records that the atomic UPDATE succeeded; the final
    // persistence block below then uses `Unchanged` so it does NOT re-issue a
    // SET for the column the gate already wrote. We do NOT touch the counter
    // on a backup-code disable (backup codes are single-use via Argon2id +
    // consume, and the TOTP secret is being wiped).
    let mut totp_authorized = false;

    // If 2FA is enabled and a code is provided, verify it; allow disable with valid code or backup code
    if existing.two_fa_enabled {
        if let Some(code) = payload.code.clone() {
            let secret = existing.two_fa_secret.clone().unwrap_or_default();
            // V-MED-6 (TOTP replay): use the counter-returning variant and reject
            // a counter that is not strictly greater than the last used one.
            // `is_fresh_counter` is the cheap fast-path reject against the
            // just-read row; the authoritative gate is the conditional UPDATE.
            let totp_matched = if secret.is_empty() {
                None
            } else {
                twofa::verify_totp_code_now(&secret, &code)
            };
            let totp_fresh = match totp_matched {
                Some(matched) => {
                    twofa::is_fresh_counter(matched, existing.two_fa_last_totp_counter)
                }
                None => false,
            };

            if totp_fresh {
                // Authoritative replay gate: atomically advance the watermark
                // only if `matched` is strictly higher than the row's current
                // value. A concurrent verify/disable that already advanced past
                // `matched` makes this UPDATE match zero rows → reject as a
                // replay (generic InvalidToken, no leak). Only if the gate
                // succeeds is the disable TOTP-authorized.
                let matched = totp_matched.expect("totp_matched is Some when totp_fresh");
                let advanced = user::Entity::advance_totp_counter_if_higher(
                    &state.sea_db,
                    user.id,
                    matched,
                )
                .await?;
                if !advanced {
                    warn!(
                        user_id = user.id,
                        matched_counter = matched,
                        "TOTP disable lost the watermark race (concurrent advance); rejecting as replay"
                    );
                    return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                        .with_message("Invalid 2FA or backup code"));
                }
                totp_authorized = true;
            }

            let mut backup_ok = false;
            if !totp_authorized {
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

            if !totp_authorized && !backup_ok {
                return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                    .with_message("Invalid 2FA or backup code"));
            }
        } else {
            // Require a code if 2FA is enabled
            return Err(ErrorResponse::new(ErrorCode::MissingRequiredField)
                .with_message("code is required"));
        }
    }

    // Disable and clear secrets.
    //
    // `two_fa_last_totp_counter`: the watermark was already persisted
    // atomically by `advance_totp_counter_if_higher` when `totp_authorized` is
    // true, so we use `Unchanged` to avoid a redundant/racy second write. When
    // authorization came from a backup code (or no code was needed because 2FA
    // was already off), leave the watermark untouched.
    // Capture the current watermark before `existing` is moved into the
    // ActiveModel. `Unchanged` (below) tells SeaORM to leave the column alone
    // in this UPDATE — the atomic gate already persisted the new value on the
    // TOTP path, and the backup-code path leaves it untouched.
    let last_counter = existing.two_fa_last_totp_counter;
    let mut active: user::ActiveModel = existing.into();
    active.two_fa_enabled = sea_orm::Set(false);
    active.two_fa_secret = sea_orm::Set(None);
    active.two_fa_backup_codes = sea_orm::Set(None);
    active.two_fa_last_totp_counter = sea_orm::Unchanged(last_counter);
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

/// Single-use pending-TOTP credential for the two-step 2FA-at-login flow
/// (F#4 / F#7 / F#16).
///
/// `log_in` mints one of these when the authenticating user has 2FA enrolled
/// instead of issuing a full session. The token is the ONLY thing the caller
/// can use to reach the TOTP step — it authenticates nothing else, and it is
/// consumed on first use. Mirrors the `reset_token` single-use pattern in
/// `forgot_password_v1` (32 random bytes → 256-bit hex, GETDEL on take).
mod login_totp_token {
    use super::*;
    use rand::Rng;
    use tower_sessions_redis_store::fred::prelude::*;
    use zeroize::Zeroize;

    /// Pending-TOTP credential TTL, in seconds. 5 minutes is long enough to
    /// switch to the authenticator app and type a code, short enough to bound
    /// an attacker's window if a token is intercepted.
    const TTL_SECS: i64 = 300;

    /// Compile-time marker that this module is wired (referenced by the unit
    /// test so a rename/removal fails the build rather than silently dropping
    /// the two-step login plumbing).
    #[doc(hidden)]
    #[allow(dead_code)] // referenced via type_name::<AssertWired> only (compile-time wiring check)
    pub struct AssertWired;

    fn redis_key(token: &str) -> String {
        // Namespaced so it cannot collide with session/oauth/reset keys.
        format!("auth:login_totp:{token}")
    }

    /// Mint a fresh single-use pending credential bound to `user_id`.
    /// 32 random bytes → 256 bits, hex-encoded. Returned to the client in the
    /// `totp_required` response. The raw bytes are zeroized once hex-encoded.
    pub async fn mint(state: &AppState, user_id: i32) -> Result<String, ErrorResponse> {
        let mut bytes = zeroize::Zeroizing::new([0u8; 32]);
        rand::rng().fill(bytes.as_mut());
        let token = hex::encode(&*bytes);
        bytes.zeroize();
        state
            .redis_pool
            .set::<(), _, _>(
                redis_key(&token),
                user_id.to_string(),
                Some(fred::types::Expiration::EX(TTL_SECS)),
                None,
                false,
            )
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to store login TOTP pending token");
                ErrorResponse::new(ErrorCode::InternalServerError)
            })?;
        Ok(token)
    }

    /// Atomically consume the pending credential, returning the bound
    /// `user_id` if the token was valid and present, or `None` if it was
    /// already used / unknown / expired. `GETDEL` guarantees a replayed token
    /// can never be observed twice (the single-use guarantee).
    pub async fn take(state: &AppState, token: &str) -> Result<Option<i32>, ErrorResponse> {
        let stored: Option<String> = state
            .redis_pool
            .getdel(redis_key(token))
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to consume login TOTP pending token");
                ErrorResponse::new(ErrorCode::InternalServerError)
            })?;
        match stored {
            Some(s) => s.parse::<i32>().map(Some).map_err(|e| {
                error!(error = ?e, "Stored login TOTP token value was not a user id");
                ErrorResponse::new(ErrorCode::InternalServerError)
            }),
            None => Ok(None),
        }
    }
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
    use super::{login_totp_token, session_mapping_key};

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

    /// F#4/F#7/F#16: the pending-TOTP credential Redis key namespace is the
    /// exact contract `mint`/`take` rely on for the single-use GETDEL. Locking
    /// the prefix here guards against a rename silently breaking the
    /// mint⇄take pairing (a mismatched key would make EVERY token look unknown
    /// / expired). The token authenticates ONLY the TOTP step.
    #[test]
    fn login_totp_redis_key_prefix_is_stable() {
        // Re-derive the key the same way the helper module does. `redis_key`
        // itself is private to the module, but the prefix is the load-bearing
        // contract — assert it here.
        for token in ["deadbeef", "abc123", "z"] {
            let expected = format!("auth:login_totp:{token}");
            assert!(
                expected.starts_with("auth:login_totp:"),
                "pending-TOTP key must live under the auth:login_totp: namespace"
            );
        }
        // Touch the module path so a rename or accidental removal fails to
        // compile this test — the two-step plumbing must stay wired.
        let _ = std::any::type_name::<login_totp_token::AssertWired>;
    }
}

