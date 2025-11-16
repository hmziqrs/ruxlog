use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_client_ip::ClientIp;
use axum_macros::debug_handler;

use serde_json::json;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{forgot_password, user},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::abuse_limiter,
    AppState,
};

use super::validator::{V1GeneratePayload, V1ResetPayload, V1VerifyPayload};

const ABUSE_LIMITER_CONFIG: abuse_limiter::AbuseLimiterConfig = abuse_limiter::AbuseLimiterConfig {
    temp_block_attempts: 3,
    temp_block_range: 360,
    temp_block_duration: 3600,
    block_retry_limit: 5,
    block_range: 900,
    block_duration: 86400,
};

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
            return Err(err.into());
        }
    }

    // Verify user exists
    match user::Entity::find_by_email(&state.sea_db, payload.email.clone()).await {
        Ok(Some(_)) => (),
        Ok(None) => {
            warn!("Forgot password requested for non-existent email");
            return Err(
                ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Email doesn't exist")
            );
        }
        Err(err) => {
            error!("Database error finding user: {}", err);
            return Err(err.into());
        }
    }

    // Send recovery email via Supabase
    match state.supabase.send_recovery_email(&payload.email).await {
        Ok(_) => {
            info!(email = %payload.email, "Recovery email sent");
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Verification code sent to your email successfully",
                })),
            ))
        }
        Err(e) => {
            error!(email = %payload.email, "Supabase recovery email failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to send recovery email"))
        }
    }
}

#[debug_handler]
#[instrument(skip(state, payload), fields(email = %payload.email))]
pub async fn verify(
    state: State<AppState>,
    payload: ValidatedJson<V1VerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Verify OTP with Supabase
    match state
        .supabase
        .verify_otp(&payload.email, &payload.code, "recovery")
        .await
    {
        Ok(_) => {
            info!(email = %payload.email, "Forgot password code verified");
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "code verified successfully",
                })),
            ))
        }
        Err(e) => {
            warn!(email = %payload.email, "Supabase OTP verification failed: {}", e);
            Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("The provided verification code is invalid"))
        }
    }
}

#[debug_handler]
#[instrument(skip(state, payload), fields(email = %payload.email))]
pub async fn reset(
    state: State<AppState>,
    payload: ValidatedJson<V1ResetPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    if payload.password != payload.confirm_password {
        warn!(email = %payload.email, "Password mismatch");
        return Err(ErrorResponse::new(ErrorCode::InvalidInput)
            .with_message("Password and confirm password do not match"));
    }

    // Verify OTP with Supabase first
    let supabase_user_id = match state
        .supabase
        .verify_otp(&payload.email, &payload.code, "recovery")
        .await
    {
        Ok(user_id) => user_id,
        Err(e) => {
            warn!(email = %payload.email, "Supabase OTP verification failed: {}", e);
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("The provided verification code is invalid"));
        }
    };

    // Find user in PostgreSQL
    let user = match user::Entity::find_by_email(&state.sea_db, payload.email.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!(email = %payload.email, "User not found");
            return Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("User not found"));
        }
        Err(err) => {
            error!(email = %payload.email, "Database error: {}", err);
            return Err(err.into());
        }
    };

    // Reset password in PostgreSQL
    match forgot_password::Entity::reset(&state.sea_db, user.id, payload.password.clone()).await {
        Ok(_) => {
            info!(user_id = user.id, email = %payload.email, "Password reset in PostgreSQL");

            // Update password in Supabase (non-blocking)
            let supabase = state.supabase.clone();
            let password = payload.password.clone();
            tokio::spawn(async move {
                match supabase
                    .admin_update_password(&supabase_user_id, &password)
                    .await
                {
                    Ok(_) => tracing::info!("Supabase password updated"),
                    Err(e) => tracing::error!("Failed to update Supabase password: {}", e),
                }
            });

            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Password reset successfully",
                })),
            ))
        }
        Err(err) => {
            error!(user_id = user.id, "Failed to reset password: {}", err);
            Err(err.into())
        }
    }
}
