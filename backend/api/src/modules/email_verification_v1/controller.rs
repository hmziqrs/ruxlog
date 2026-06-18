use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{email_verification, user},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::{abuse_limiter, auth::AuthSession, mail::send_email_verification_code},
    AppState,
};

use super::validator::V1VerifyPayload;

const ABUSE_LIMITER_CONFIG: abuse_limiter::AbuseLimiterConfig = abuse_limiter::AbuseLimiterConfig {
    temp_block_attempts: 3,
    temp_block_range: 360,
    temp_block_duration: 3600,
    block_retry_limit: 5,
    block_range: 900,
    block_duration: 86400,
};

#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id = auth.user.as_ref().map(|u| u.id), code = %payload.code))]
pub async fn verify(
    state: State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1VerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let user_id = user.id;

    // Throttle code-guessing per account. Fail-closed: a Redis outage denies
    // the attempt rather than allowing unbounded tries.
    let key_prefix = format!("email_verify:{}", user_id);
    abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await?;

    let code = payload.0.code;
    let code_hash = crate::utils::code_hash::hash_code(&state.secret_key, &code);

    let verification_result = email_verification::Entity::find_by_user_id_or_code(
        &state.sea_db,
        Some(user_id),
        Some(code_hash),
    )
    .await;

    match verification_result {
        Ok(verification) => {
            if verification.is_expired() {
                warn!(user_id, "Email verification code expired");
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
        }
        Err(err) => {
            warn!(user_id, "Invalid email verification code");
            return Err(err);
        }
    }

    match user::Entity::verify(&state.sea_db, user_id).await {
        Ok(_) => {
            // Single-use: delete the verification row so the (hash of the)
            // code cannot be replayed. Audit: "codes not consumed at verify"
            // — fixed in Phase 3d.
            if let Err(err) = email_verification::Entity::consume(&state.sea_db, user_id).await {
                warn!(user_id, "Failed to consume verification code: {}", err);
            }
            info!(user_id, "Email verified successfully");
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Email verified successfully",
                })),
            ))
        }
        Err(err) => {
            error!(
                user_id,
                "Failed to update user verification status: {}", err
            );
            Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to update user verification status")
                .with_details(err.to_string()))
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth), fields(user_id = auth.user.as_ref().map(|u| u.id)))]
pub async fn resend(
    state: State<AppState>,
    auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pool = &state.sea_db;
    let user = auth.user.unwrap();
    let user_id = user.id;

    match email_verification::Entity::find_by_user_id_or_code(pool, Some(user_id), None).await {
        Ok(verification) => {
            if verification.is_in_delay() {
                warn!(user_id, "Email verification resend in delay period");
                return Err(ErrorResponse::new(ErrorCode::TooManyAttempts).with_message(
                    "Please wait 1 minute before requesting a new verification code",
                ));
            }
        }
        Err(err) => {
            if err.code != ErrorCode::InvalidInput {
                error!(user_id, "Error checking verification delay: {}", err);
                return Err(err);
            }
        }
    }

    // Rate limiting via abuse limiter (3 attempts per 6 minutes)
    let key_prefix = format!("email_verification:{}", user_id);
    match abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await {
        Ok(_) => (),
        Err(err) => {
            warn!(user_id, "Abuse limiter blocked verification resend");
            return Err(err);
        }
    }

    // Generate a fresh plaintext code, store only its keyed hash, and email the
    // plaintext. The plaintext never touches the database (audit: "brute-forceable
    // plaintext verification codes" — fixed in Phase 3d).
    let code = email_verification::Entity::generate_code();
    let code_hash = crate::utils::code_hash::hash_code(&state.secret_key, &code);
    email_verification::Entity::regenerate(pool, user_id, code_hash).await?;
    if let Err(err) = send_email_verification_code(&state.mailer, &user.email, &code).await {
        error!(user_id, "Failed to send verification email: {}", err);
        return Err(ErrorResponse::new(ErrorCode::ExternalServiceError)
            .with_message("Failed to send verification email")
            .with_details(err));
    }

    info!(user_id, "Verification email sent");
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Verification email sent",
        })),
    ))
}
