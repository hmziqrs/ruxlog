use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::sea_models::{email_verification, user}, 
    error::{ErrorCode, ErrorResponse}, 
    extractors::ValidatedJson, 
    services::{abuse_limiter, auth::AuthSession}, 
    AppState
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
pub async fn verify(
    state: State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1VerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user_id = auth.user.unwrap().id;
    let code = payload.0.code;

    let verification_result = email_verification::Entity::find_by_user_id_and_code(
        &state.sea_db,
        user_id,
        code,
    )
    .await;

    match verification_result {
        Ok(verification) => match verification {
            Some(verification) => {
                if verification.is_expired() {
                    return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                        .with_message("The verification code has expired"));
                }
            }
            None => {
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The provided verification code is invalid"));
            }
        },
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("The provided verification code is invalid")
                .with_details(err.to_string()));
        }
    }

    let update_user = user::Entity::verify(&state.sea_db, user_id).await;
    match update_user {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "message": "Email verified successfully",
            })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to update user verification status")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn resend(state: State<AppState>, auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    let pool = &state.sea_db;
    let user_id = auth.user.unwrap().id;

    match email_verification::Entity::find_by_user_id_or_code(pool, Some(user_id), None).await {
        Ok(verification) => {
            if let Some(verification) = verification {
                if verification.is_in_delay() {
                    return Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
                        .with_message("Please wait 1 minute before requesting a new verification code"));
                }
            }
        },
        Err(err) => {
            return Err(err.into());
        }
    }

    let key_prefix = format!("email_verification:{}", user_id);
    match abuse_limiter::limiter(&state.redis_pool, &key_prefix, ABUSE_LIMITER_CONFIG).await {
        Ok(_) => (),
        Err(err) => return Err(err.into()),
    }
    match email_verification::Entity::regenerate(pool, user_id).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "message": "Verification code resent successfully",
            })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to resend verification code")
            .with_details(err.to_string())),
    }
}
