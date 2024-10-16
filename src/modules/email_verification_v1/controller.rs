use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::{email_verification::EmailVerification, user::User},
    services::auth::AuthSession,
    AppState,
};

use super::validator::V1VerifyPayload;

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
