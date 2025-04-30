use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_client_ip::ClientIp;
use axum_macros::debug_handler;

use serde_json::json;

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
pub async fn generate(
    state: State<AppState>,
    ClientIp(secure_ip): ClientIp,
    payload: ValidatedJson<V1GeneratePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let ip = secure_ip.to_string();
    let key_prefix = format!("forgot_password:{}", ip);
    match abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await {
        Ok(_) => (),
        Err(err) => return Err(err.into()),
    }

    let pool = &state.sea_db;
    let user = user::Entity::find_by_email(pool, payload.email.clone()).await;

    match user {
        Ok(Some(_)) => (),
        Ok(None) => {
            return Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Email doesn't exist"));
        }
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Request failed")
                .with_details(err.to_string()));
        }
    }
    let user_id = user.unwrap().unwrap().id;

    match forgot_password::Entity::find_by_user_id(pool, user_id.clone()).await {
        Ok(Some(verification)) => {
            if verification.is_in_delay() {
                return Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
                    .with_message("You have already requested a verification code. Please try again after 1 minute"));
            }
        }
        Ok(_) => (),
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to resend verification code")
                .with_details(err.to_string()));
        }
    }

    let result = forgot_password::Entity::generate(pool, user_id).await;

    match result {
        Ok(result) => {
            let mailer = &state.mailer;
            let code = result.code;

            match send_forgot_password_email(&mailer, "test@hello.xyz", &code).await {
                Ok(()) => {
                    return Ok((
                        StatusCode::OK,
                        Json(json!({
                            "message": "Verification code sent to your email successfully",
                        })),
                    ));
                }
                Err(err) => {
                    return Err(ErrorResponse::new(ErrorCode::ExternalServiceError)
                        .with_message("Failed to send verification code")
                        .with_details(err.to_string()));
                }
            }
        }
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to send verification code")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn verify(
    state: State<AppState>,
    payload: ValidatedJson<V1VerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let pool = &state.sea_db;

    let result = user::Entity::find_by_email_and_forgot_password(
        pool,
        payload.email.clone(),
        payload.code.clone(),
    )
    .await;

    match result {
        Ok(Some((_, verification))) => {
            if verification.is_expired() {
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
        }
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Email or code is invalid")
                .with_details(err.to_string()));
        }
        Ok(None) => {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Email or code is invalid"));
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "code verified successfully",
        })),
    ))
}

#[debug_handler]
pub async fn reset(
    state: State<AppState>,
    payload: ValidatedJson<V1ResetPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    if payload.password != payload.confirm_password {
        return Err(ErrorResponse::new(ErrorCode::InvalidInput)
            .with_message("Password and confirm password do not match"));
    }

    let pool = &state.sea_db;

    let result = user::Entity::find_by_email_and_forgot_password(
        pool,
        payload.email.clone(),
        payload.code.clone(),
    )
    .await;

    match &result {
        Ok(Some((_, verification))) => {
            if verification.is_expired() {
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
        }
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Email or code is invalid")
                .with_details(err.to_string()));
        }
        Ok(None) => {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Email or code is invalid"));
        }
    }
    // SAFETY: `result` is checked to be `Some` above
    let user_id = result.unwrap().unwrap().0.id;
    match user::Entity::change_password(pool, user_id, payload.password.clone()).await {
        Ok(_) => {
            return Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Password reset successfully",
                })),
            ));
        }
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to reset password")
                .with_details(err.to_string()));
        }
    }
}
