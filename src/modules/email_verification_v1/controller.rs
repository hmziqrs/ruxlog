use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::{email_verification::EmailVerification, user::User},
    services::auth::AuthSession,
    AppState,
};

use super::{abuse_limiter::email_abuse_limiter, validator::V1VerifyPayload};

#[debug_handler]
pub async fn verify(
    state: State<AppState>,
    auth: AuthSession,
    WithValidation(payload): WithValidation<Json<V1VerifyPayload>>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let user_id = auth.user.unwrap().id;

    match EmailVerification::find_by_user_id_and_code(pool, user_id, &payload.code).await {
        Ok(email_verification) => {
            // Check if the code is expired
            if email_verification.is_expired() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Code expired",
                        "message": "The verification code has expired",
                    })),
                )
                    .into_response();
            };

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
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid code",
                "message": "The provided verification code is invalid",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn resend(state: State<AppState>, auth: AuthSession) -> impl IntoResponse {
    let pool = &state.db_pool;
    let user_id = auth.user.unwrap().id;

    match EmailVerification::find_by_user_id(pool, user_id).await {
        Ok(verification) => {
            if verification.is_in_delay() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Request failed",
                        "message": "Please wait before requesting a new verification code",
                    })),
                )
                    .into_response();
            }
            match email_abuse_limiter(&state.redis_pool, &user_id).await {
                Ok(_) => (),
                Err(response) => return response,
            }
            match EmailVerification::regenerate(pool, user_id).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({
                        "message": "Verification code resent successfully",
                    })),
                )
                    .into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": err.to_string(),
                        "message": "Failed to resend verification code",
                    })),
                )
                    .into_response(),
            }
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Request failed",
            })),
        )
            .into_response(),
    }
}
