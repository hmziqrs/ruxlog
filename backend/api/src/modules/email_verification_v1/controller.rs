use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{email_verification, user},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::{abuse_limiter, auth::AuthSession},
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
    let code = payload.0.code;

    // Verify OTP with Supabase
    match state
        .supabase
        .verify_otp(&user.email, &code, "email")
        .await
    {
        Ok(_) => {
            // Mark user as verified in PostgreSQL
            match user::Entity::verify(&state.sea_db, user.id).await {
                Ok(_) => {
                    info!(user_id = user.id, "Email verified successfully");
                    Ok((
                        StatusCode::OK,
                        Json(json!({
                            "message": "Email verified successfully",
                        })),
                    ))
                }
                Err(err) => {
                    error!(
                        user_id = user.id,
                        "Failed to update user verification status: {}", err
                    );
                    Err(ErrorResponse::new(ErrorCode::InternalServerError)
                        .with_message("Failed to update user verification status")
                        .with_details(err.to_string()))
                }
            }
        }
        Err(e) => {
            warn!(user_id = user.id, "Supabase OTP verification failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("The provided verification code is invalid"))
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

    // Check 1-minute delay
    match email_verification::Entity::find_by_user_id_or_code(pool, Some(user.id), None).await {
        Ok(verification) => {
            if verification.is_in_delay() {
                warn!(user_id = user.id, "Email verification resend in delay period");
                return Err(ErrorResponse::new(ErrorCode::TooManyAttempts).with_message(
                    "Please wait 1 minute before requesting a new verification code",
                ));
            }
        }
        Err(err) => {
            error!(user_id = user.id, "Error checking verification delay: {}", err);
            return Err(err.into());
        }
    }

    // Rate limiting
    let key_prefix = format!("email_verification:{}", user.id);
    match abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await {
        Ok(_) => (),
        Err(err) => {
            warn!(user_id = user.id, "Abuse limiter blocked verification resend");
            return Err(err.into());
        }
    }

    // Resend via Supabase
    match state.supabase.resend_verification(&user.email).await {
        Ok(_) => {
            // Update timestamp in PostgreSQL for rate limiting
            match email_verification::Entity::regenerate(pool, user.id).await {
                Ok(_) => {
                    info!(user_id = user.id, "Verification email sent");
                    Ok((
                        StatusCode::OK,
                        Json(json!({
                            "message": "Verification email sent",
                        })),
                    ))
                }
                Err(err) => {
                    error!(user_id = user.id, "Failed to update verification timestamp: {}", err);
                    Err(ErrorResponse::new(ErrorCode::InternalServerError)
                        .with_message("Failed to send verification email")
                        .with_details(err.to_string()))
                }
            }
        }
        Err(e) => {
            error!(user_id = user.id, "Supabase resend failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to send verification email"))
        }
    }
}
