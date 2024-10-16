use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use super::validator::V1UpdateProfilePayload;

use crate::{
    db::models::user::{UpdateUser, User},
    services::auth::AuthSession,
    AppState,
};

#[debug_handler]
pub async fn get_profile(auth: AuthSession) -> impl IntoResponse {
    match auth.user {
        Some(user) => (StatusCode::OK, Json(json!({"data": user}))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "message": "No user with this ID exists",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn update_profile(
    auth: AuthSession,
    state: State<AppState>,
    WithValidation(payload): WithValidation<Json<V1UpdateProfilePayload>>,
) -> impl IntoResponse {
    if let Some(user) = auth.user {
        let payload = payload.into_inner();
        let updated_user = User::update(
            &state.db_pool,
            user.id,
            UpdateUser {
                name: payload.name,
                email: payload.email,
                password: payload.password,
            },
        )
        .await;
        match updated_user {
            Ok(user) => {
                return (StatusCode::OK, Json(json!(user))).into_response();
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": err.to_string(),
                        "message": "An error occurred while updating the user profile",
                    })),
                )
                    .into_response();
            }
        }
    }
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Unauthorized",
            "message": "You must be logged in to access this resource",
        })),
    )
        .into_response()
}
