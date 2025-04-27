use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_client_ip::ClientIp;
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use crate::{
    db::models::{forgot_password::ForgotPassword, user::User},
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
) -> impl IntoResponse {
    let ip = secure_ip.to_string();
    let key_prefix = format!("forgot_password:{}", ip);
    match abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await {
        Ok(_) => (),
        Err(response) => return response,
    }

    let pool = &state.db_pool;
    let user = User::find_by_email(pool, payload.email.clone()).await;

    match user {
        Ok(Some(_)) => (),
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Request failed",
                    "message": "email doesn't exist",
                })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": err.to_string(),
                    "message": "Request failed",
                })),
            )
                .into_response();
        }
    }
    let user_id = user.unwrap().unwrap().id;

    match ForgotPassword::find_by_user_id(pool, user_id.clone()).await {
        Ok(Some(verification)) => {
            if verification.is_in_delay() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Request failed",
                        "message": "You have already requested a verification code. Please try again after 1 minute",
                    })),
                )
                    .into_response();
            }
        }
        Ok(_) => (),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": err.to_string(),
                    "message": "Failed to resend verification code",
                })),
            )
                .into_response();
        }
    }

    let result = ForgotPassword::generate(pool, user_id).await;

    match result {
        Ok(result) => {
            let mailer = &state.mailer;
            let code = result.code;

            match send_forgot_password_email(&mailer, "test@hello.xyz", &code).await {
                Ok(()) => {
                    return (
                        StatusCode::OK,
                        Json(json!({
                            "message": "Verification code sent to your email successfully",
                        })),
                    )
                        .into_response();
                }
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": err.to_string(),
                            "message": "Failed to send verification code",
                        })),
                    )
                        .into_response();
                }
            }
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to send verification code",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn verify(
    state: State<AppState>,
    payload: ValidatedJson<V1VerifyPayload>,
) -> impl IntoResponse {
    let pool = &state.db_pool;

    let result =
        User::find_by_email_and_forgot_password(pool, payload.email.clone(), payload.code.clone())
            .await;

    match result {
        Ok(Some((_, verification))) => {
            if verification.is_expired() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Code expired",
                        "message": "The verification code has expired",
                    })),
                )
                    .into_response();
            }
        }
        Err(_) | Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid request",
                    "message": "Email or code is invalid",
                })),
            )
                .into_response();
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "code verified successfully",
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn reset(
    state: State<AppState>,
    payload: ValidatedJson<V1ResetPayload>,
) -> impl IntoResponse {
    if payload.password != payload.confirm_password {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid request",
                "message": "Password and confirm password do not match",
            })),
        )
            .into_response();
    }

    let pool = &state.db_pool;

    let result =
        User::find_by_email_and_forgot_password(pool, payload.email.clone(), payload.code.clone())
            .await;

    match &result {
        Ok(Some((_, verification))) => {
            if verification.is_expired() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Code expired",
                        "message": "The verification code has expired",
                    })),
                )
                    .into_response();
            }
        }
        Err(_) | Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid request",
                    "message": "Email or code is invalid",
                })),
            )
                .into_response();
        }
    }
    // SAFETY: `result` is checked to be `Some` above
    let user_id = result.unwrap().unwrap().0.id;
    match User::change_password(pool, user_id, payload.password.clone()).await {
        Ok(_) => {
            return (
                StatusCode::OK,
                Json(json!({
                    "message": "Password reset successfully",
                })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": err.to_string(),
                    "message": "Failed to reset password",
                })),
            )
                .into_response();
        }
    }
}
