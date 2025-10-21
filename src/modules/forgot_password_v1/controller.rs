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
            return Err(
                ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Email doesn't exist")
            );
        }
        Err(err) => {
            return Err(err.into());
        }
    }
    let user_id = user.unwrap().unwrap().id;

    match forgot_password::Entity::find_query(pool, Some(user_id), None, None).await {
        Ok(verification) => {
            if verification.is_in_delay() {
                return Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
                    .with_message("You have already requested a verification code. Please try again after 1 minute"));
            }
        }
        Err(err) => {
            return Err(err.into());
        }
    }

    let result = forgot_password::Entity::regenerate(pool, user_id).await;

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
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
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
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
        }
        Err(err) => {
            return Err(err.into());
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

    let result = forgot_password::Entity::find_query(
        &state.sea_db,
        None,
        Some(&payload.email),
        Some(&payload.code),
    )
    .await;

    match &result {
        Ok(verification) => {
            if verification.is_expired() {
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The verification code has expired"));
            }
        }
        Err(err) => {
            return Err(err.to_owned().into());
        }
    }
    let res = result.unwrap();
    match forgot_password::Entity::reset(&state.sea_db, res.user_id, payload.password.clone()).await
    {
        Ok(_) => {
            return Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Password reset successfully",
                })),
            ));
        }
        Err(err) => {
            return Err(err.into());
        }
    }
}
