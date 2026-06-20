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
    // GAP-016: `Zeroize` (the trait) must be in scope for the `.zeroize()`
    // call on the `Zeroizing<[u8;32]>` random-source wrapper below.
    use zeroize::Zeroize;

    fn redis_key(token: &str) -> String {
        // Namespaced so it can't collide with session/checkout/oauth keys.
        format!("forgot_password:reset_token:{token}")
    }

    /// Mint a fresh single-use token bound to `user_id`. 32 random bytes → 256
    /// bits, hex-encoded. Returned to the client in the `verify` response.
    /// 10-minute TTL: longer than a user spends typing a new password, short
    /// enough to bound an attacker's replay window.
    ///
    /// GAP-016 (CWE-316/459): the 32-byte random source is zeroized the moment
    /// it has been hex-encoded, so the raw token material is not left on the
    /// stack/heap longer than necessary. The hex `String` itself is returned to
    /// the caller (it MUST reach the client JSON), so it is not zeroized here —
    /// its lifetime is bounded by the short Redis TTL and the single-use GETDEL
    /// on consumption.
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

/// The single, uniform success response returned by `generate` for EVERY
/// `/request` call — whether or not the supplied email corresponds to a real
/// account (SC-006). Returning the identical body/status for a known vs
/// unknown email closes the user-enumeration oracle the old `RecordNotFound`
/// ("Email doesn't exist") branch opened.
///
/// Kept as a free function (rather than inlined) so a unit test can assert both
/// code paths produce byte-identical output without a DB.
pub(crate) fn uniform_success_response(
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "If an account exists for that email, a password reset link has been sent.",
        })),
    )
}

#[debug_handler]
#[instrument(skip(state, payload), fields(email = %payload.email, client_ip = %secure_ip))]
pub async fn generate(
    state: State<AppState>,
    ClientIp(secure_ip): ClientIp,
    payload: ValidatedJson<V1GeneratePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Rate limiting via abuse limiter (3 attempts per 6 minutes). Kept
    // unconditionally (SC-006: do NOT weaken abuse protection) and run BEFORE
    // the existence check, so an attacker probing emails is throttled just as
    // a legitimate user is.
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
    // SC-006: an UNKNOWN email must produce the SAME response as a known one.
    // We do NOT error, and we do NOT send an email — we just short-circuit to
    // the uniform success body. Only genuine infrastructure failures (DB down)
    // still surface as a 500; a normal "no such user" is indistinguishable to
    // the caller from a successful code dispatch.
    let user = match user::Entity::find_by_email(pool, payload.email.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Forgot password requested for non-existent email; returning uniform response");
            return Ok(uniform_success_response());
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
    // SC-006: identical body/status to the unknown-email branch above. The
    // message deliberately does NOT confirm the email exists.
    Ok(uniform_success_response())
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

#[cfg(test)]
mod tests {
    use super::*;

    // SC-006: the `/request` handler must NOT leak whether an email is
    // registered. Both the known-email success path and the unknown-email
    // short-circuit return `uniform_success_response()`, so an attacker probing
    // emails cannot distinguish a hit from a miss by body or status. We assert
    // this at the unit level by calling the shared helper twice (it is the
    // single source of the response for both branches) and proving the outputs
    // are byte-identical — i.e. the same constant shape is used, and it carries
    // no existence signal.
    #[test]
    fn uniform_success_response_is_stable_and_non_leaking() {
        let (status_known, body_known) = uniform_success_response();
        let (status_unknown, body_unknown) = uniform_success_response();

        // 1. Same HTTP status for both branches.
        assert_eq!(status_known, status_unknown);
        assert_eq!(status_known, StatusCode::OK);

        // 2. Byte-identical JSON bodies.
        let known = serde_json::to_value(&*body_known).unwrap();
        let unknown = serde_json::to_value(&*body_unknown).unwrap();
        assert_eq!(known, unknown);

        // 3. The message must NOT confirm existence (no "doesn't exist",
        //    no "sent successfully", no "verified"). It must be conditional.
        let msg = known["message"].as_str().unwrap().to_lowercase();
        assert!(
            msg.contains("if an account exists"),
            "uniform message must be conditional, got: {msg}"
        );
        assert!(
            !msg.contains("doesn't exist") && !msg.contains("does not exist"),
            "uniform message must not leak non-existence, got: {msg}"
        );
        assert!(
            !msg.contains("sent successfully"),
            "uniform message must not confirm a send, got: {msg}"
        );
    }

    // SC-006 regression guard: the old leak shape ("Email doesn't exist",
    // status from ErrorCode::RecordNotFound) must NOT match the uniform
    // response. If a future edit accidentally re-introduces the oracle, this
    // test fails — the uniform body must never equal an error body.
    #[test]
    fn uniform_response_differs_from_old_record_not_found_leak() {
        let (status, body) = uniform_success_response();
        let leak = ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Email doesn't exist");

        // The success status must not be the leak's status.
        assert_ne!(status, StatusCode::NOT_FOUND);
        // And the bodies must differ (one is a success JSON, the other an error).
        let success = serde_json::to_value(&*body).unwrap();
        let leak_body = serde_json::to_value(&leak).unwrap_or(serde_json::Value::Null);
        assert_ne!(success, leak_body);
    }
}
