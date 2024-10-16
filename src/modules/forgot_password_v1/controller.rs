use std::fmt::format;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_client_ip::{InsecureClientIp, SecureClientIp};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::{
        email_verification::EmailVerification, forgot_password::ForgotPassword, user::User,
    },
    services::{abuse_limiter, auth::AuthSession},
    AppState,
};

use super::validator::{V1GeneratePayload, V1VerifyPayload};

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
    secure_ip: SecureClientIp,
    WithValidation(payload): WithValidation<Json<V1GeneratePayload>>,
) -> impl IntoResponse {
    let ip = secure_ip.0.to_string();
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

    match ForgotPassword::generate(pool, user_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "message": "Verification code sent to your email successfully",
            })),
        )
            .into_response(),
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
    auth: AuthSession,
    WithValidation(payload): WithValidation<Json<V1VerifyPayload>>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let user_id = auth.user.unwrap().id;

    let verification_result =
        EmailVerification::find_by_user_id_and_code(pool, user_id, &payload.code).await;

    match verification_result {
        Ok(verification) => match verification {
            Some(verification) => {
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
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Code not found",
                        "message": "The provided verification code is invalid",
                    })),
                )
                    .into_response();
            }
        },
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid code",
                    "message": "The provided verification code is invalid",
                })),
            )
                .into_response();
        }
    }

    let update_user = User::verify(pool, user_id).await;
    match update_user {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "message": "Email verified successfully",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to update user verification status",
            })),
        )
            .into_response(),
    }
}
