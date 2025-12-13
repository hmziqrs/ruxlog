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
            return Err(err.into());
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
                return Err(err.into());
            }
        }
    }

    let result = forgot_password::Entity::regenerate(pool, user_id).await?;
    let code = result.code;
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
#[instrument(skip(state, payload), fields(email = %payload.email))]
pub async fn verify(
    state: State<AppState>,
    payload: ValidatedJson<V1VerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = forgot_password::Entity::find_query(
        &state.sea_db,
        None,
        Some(&payload.email),
        Some(&payload.code),
    )
    .await;

    match result {
        Ok(verification) => {
            if verification.is_expired() {
                warn!(email = %payload.email, "Forgot password code expired");
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
        }
        Err(err) => {
            warn!(email = %payload.email, "Invalid forgot password code");
            return Err(err.into());
        }
    }

    info!(email = %payload.email, "Forgot password code verified");
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "code verified successfully",
        })),
    ))
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

    let result = forgot_password::Entity::find_query(
        &state.sea_db,
        None,
        Some(&payload.email),
        Some(&payload.code),
    )
    .await;

    let verification = match &result {
        Ok(verification) => {
            if verification.is_expired() {
                warn!(email = %payload.email, "Expired code during reset");
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
            verification
        }
        Err(err) => {
            warn!(email = %payload.email, "Invalid code during reset");
            return Err(err.to_owned().into());
        }
    };
    let user_id = verification.user_id;

    // Reset password in PostgreSQL
    match forgot_password::Entity::reset(&state.sea_db, user_id, payload.password.clone()).await {
        Ok(_) => {
            info!(user_id, email = %payload.email, "Password reset in PostgreSQL");

            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Password reset successfully",
                })),
            ))
        }
        Err(err) => {
            error!(user_id, "Failed to reset password: {}", err);
            Err(err.into())
        }
    }
}
