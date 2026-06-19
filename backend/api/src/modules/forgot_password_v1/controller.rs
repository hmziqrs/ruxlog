use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_client_ip::ClientIp;
use axum_macros::debug_handler;

use serde_json::json;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{forgot_password, user},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::{abuse_limiter, mail::send_forgot_password_email},
    AppState,
};

use super::validator::{V1GeneratePayload, V1ResetPayload, V1VerifyPayload, V1VerifyResponse};

const ABUSE_LIMITER_CONFIG: abuse_limiter::AbuseLimiterConfig = abuse_limiter::AbuseLimiterConfig {
    temp_block_attempts: 3,
    temp_block_range: 360,
    temp_block_duration: 3600,
    block_retry_limit: 5,
    block_range: 900,
    block_duration: 86400,
};

/// Single-use reset token minted by `verify` and consumed by `reset`
/// (audit F#9).
///
/// The emailed code is deleted from the DB the moment `verify` accepts it; the
/// only thing that can subsequently change the password is this opaque token,
/// which is stored in Redis with a short TTL and burned (atomic `GETDEL`) on
/// first use. Consequences:
///   - the emailed code can never be **replayed** against `verify` or `reset`
///     once the legitimate user has verified;
///   - the reset token itself is strictly single-use — a replayed `reset`
///     request can never observe the token twice.
///
/// This closes the window the old flow left open, where `verify` merely *checked*
/// the code and left it live so the (single) `reset` could re-check it — meaning
/// the code stayed reusable until consumed.
mod reset_token {
    use super::*;
    // fred's `set`/`getdel` are trait methods on the Pool.
    use rand::Rng;
    use tower_sessions_redis_store::fred::prelude::*;

    fn redis_key(token: &str) -> String {
        // Namespaced so it can't collide with session/checkout/oauth keys.
        format!("forgot_password:reset_token:{token}")
    }

    /// Mint a fresh single-use token bound to `user_id`. 32 random bytes → 256
    /// bits, hex-encoded. Returned to the client in the `verify` response.
    /// 10-minute TTL: longer than a user spends typing a new password, short
    /// enough to bound an attacker's replay window.
    pub async fn mint(state: &AppState, user_id: i32) -> Result<String, ErrorResponse> {
        let bytes: [u8; 32] = rand::rng().random();
        let token = hex::encode(bytes);
        state
            .redis_pool
            .set::<(), _, _>(
                redis_key(&token),
                user_id.to_string(),
                Some(fred::types::Expiration::EX(600)),
                None,
                false,
            )
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to store reset token in Redis");
                ErrorResponse::new(ErrorCode::InternalServerError)
            })?;
        Ok(token)
    }

    /// Atomically consume the token, returning the bound `user_id` if the token
    /// was valid and present, or `None` if it was already used / unknown / expired.
    /// `GETDEL` guarantees a replayed `reset` can never observe the same token
    /// twice (the same atomic-take guarantee used for checkout intents).
    pub async fn take(state: &AppState, token: &str) -> Result<Option<i32>, ErrorResponse> {
        let stored: Option<String> = state
            .redis_pool
            .getdel(redis_key(token))
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to consume reset token from Redis");
                ErrorResponse::new(ErrorCode::InternalServerError)
            })?;
        match stored {
            Some(s) => s.parse::<i32>().map(Some).map_err(|e| {
                error!(error = ?e, "Stored reset token value was not a user id");
                ErrorResponse::new(ErrorCode::InternalServerError)
            }),
            None => Ok(None),
        }
    }
}

#[debug_handler]
#[instrument(skip(state, payload), fields(email = %payload.email, client_ip = %secure_ip))]
pub async fn generate(
    state: State<AppState>,
    ClientIp(secure_ip): ClientIp,
    payload: ValidatedJson<V1GeneratePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Rate limiting via abuse limiter (3 attempts per 6 minutes)
    let ip = secure_ip.to_string();
    let key_prefix = format!("forgot_password:{}", ip);
    match abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await {
        Ok(_) => (),
        Err(err) => {
            warn!("Abuse limiter blocked forgot password request");
            return Err(err);
        }
    }

    let pool = &state.sea_db;
    let user = match user::Entity::find_by_email(pool, payload.email.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Forgot password requested for non-existent email");
            return Err(
                ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Email doesn't exist")
            );
        }
        Err(err) => {
            error!("Database error finding user: {}", err);
            return Err(err);
        }
    };
    let user_id = user.id;

    match forgot_password::Entity::find_query(pool, Some(user_id), None, None).await {
        Ok(verification) => {
            if verification.is_in_delay() {
                warn!(user_id, "Forgot password in delay period");
                return Err(ErrorResponse::new(ErrorCode::TooManyAttempts).with_message(
                    "You have already requested a verification code. Please try again after 1 minute",
                ));
            }
        }
        Err(err) => {
            if err.code != ErrorCode::InvalidInput {
                error!(user_id, "Error checking forgot password delay: {}", err);
                return Err(err);
            }
        }
    }

    // Generate a fresh plaintext code, store only its keyed hash, and email the
    // plaintext. The plaintext never touches the database (audit: "brute-forceable
    // plaintext reset codes" — fixed in Phase 3d).
    let code = forgot_password::Entity::generate_code();
    let code_hash = crate::utils::code_hash::hash_code(&state.secret_key, &code);
    if let Err(err) = forgot_password::Entity::regenerate(pool, user_id, code_hash).await {
        error!(user_id, email = %payload.email, "Failed to store forgot-password code: {}", err);
        return Err(err);
    }
    if let Err(err) = send_forgot_password_email(&state.mailer, &payload.email, &code).await {
        error!(user_id, email = %payload.email, "Failed to send forgot password email: {}", err);
        return Err(ErrorResponse::new(ErrorCode::ExternalServiceError)
            .with_message("Failed to send verification code")
            .with_details(err));
    }

    info!(user_id, email = %payload.email, "Recovery email sent");
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Verification code sent to your email successfully",
        })),
    ))
}

#[debug_handler]
#[instrument(skip(state, payload), fields(email = %payload.email, client_ip = %secure_ip))]
pub async fn verify(
    state: State<AppState>,
    ClientIp(secure_ip): ClientIp,
    payload: ValidatedJson<V1VerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Throttle code-guessing. Fail-closed: a Redis outage denies the attempt.
    let key_prefix = format!("forgot_password_verify:{}", secure_ip);
    abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await?;

    let code_hash = crate::utils::code_hash::hash_code(&state.secret_key, &payload.code);
    let result = forgot_password::Entity::find_query(
        &state.sea_db,
        None,
        Some(&payload.email),
        Some(&code_hash),
    )
    .await;

    let verification = match result {
        Ok(verification) => {
            if verification.is_expired() {
                warn!(email = %payload.email, "Forgot password code expired");
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
            verification
        }
        Err(err) => {
            warn!(email = %payload.email, "Invalid forgot password code");
            return Err(err);
        }
    };
    let user_id = verification.user_id;

    // Make the emailed code single-use (audit F#9): delete its row NOW so it
    // can't be replayed against `verify` or used by the legacy `reset` path.
    // The password is NOT changed here — that requires the freshly-issued
    // reset_token, which we return below.
    if let Err(err) = forgot_password::Entity::consume_code(&state.sea_db, user_id).await {
        error!(user_id, email = %payload.email, "Failed to consume forgot-password code: {}", err);
        return Err(err);
    }

    let reset_token = reset_token::mint(&state, user_id).await?;

    info!(user_id, email = %payload.email, "Forgot password code verified and consumed; reset token issued");
    Ok((
        StatusCode::OK,
        Json(V1VerifyResponse { reset_token }),
    ))
}

#[debug_handler]
#[instrument(skip(state, payload), fields(client_ip = %secure_ip))]
pub async fn reset(
    state: State<AppState>,
    ClientIp(secure_ip): ClientIp,
    payload: ValidatedJson<V1ResetPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Shares the verify bucket so an attacker can't reset-vote their way past
    // the verify limiter (or vice-versa). Fail-closed.
    let key_prefix = format!("forgot_password_verify:{}", secure_ip);
    abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await?;

    if payload.password != payload.confirm_password {
        warn!("Password mismatch");
        return Err(ErrorResponse::new(ErrorCode::InvalidInput)
            .with_message("Password and confirm password do not match"));
    }

    // V-HIGH-4: the password can ONLY be changed through the one-time
    // `reset_token` issued by `verify`. That token is bound to a user and is
    // atomically consumed here (single-use `GETDEL`). There is NO fallback to a
    // raw emailed `code` + `email`: `/request`, `/verify` and `/reset` are
    // independently-reachable routes, so accepting the emailed code at `/reset`
    // would let an attacker who merely intercepted the reset email skip
    // `/verify` entirely and take over the account. The `reset_token` is a
    // required (non-optional) field on `V1ResetPayload`, so a tokenless request
    // fails at deserialization before reaching this handler.
    let user_id = match reset_token::take(&state, &payload.reset_token).await? {
        Some(id) => id,
        None => {
            warn!("Reset attempted with an unknown or already-used reset token");
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Reset token is invalid or has expired"));
        }
    };

    // Reset password in PostgreSQL. `reset` runs in a transaction that deletes
    // any remaining code row for the user before updating the password, so the
    // row is consumed as part of this flow even if a stale one lingered.
    match forgot_password::Entity::reset(&state.sea_db, user_id, payload.password.clone()).await {
        Ok(_) => {
            info!(user_id, "Password reset in PostgreSQL");
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Password reset successfully",
                })),
            ))
        }
        Err(err) => {
            error!(user_id, "Failed to reset password: {}", err);
            Err(err)
        }
    }
}
