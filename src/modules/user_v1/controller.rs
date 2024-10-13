use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::{headers::Cookie, TypedHeader};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use super::validator::V1UpdateProfilePayload;

use crate::{
    db::models::user::{UpdateUser, User},
    AppState,
};

#[debug_handler]
pub async fn get_profile(
    state: State<AppState>,
    headers: TypedHeader<Cookie>,
) -> impl IntoResponse {
    let user_id = extract_user_id_from_headers(headers); // Placeholder function
    if let Some(user_id) = user_id {
        let user = User::find_by_id(&state.db_pool, user_id).await;
        match user {
            Ok(Some(user)) => {
                return (StatusCode::OK, Json(json!(user))).into_response();
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "User not found",
                        "message": "No user with this ID exists",
                    })),
                )
                    .into_response();
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": err.to_string(),
                        "message": "An error occurred while fetching the user",
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

#[debug_handler]
pub async fn update_profile(
    state: State<AppState>,
    headers: TypedHeader<Cookie>,
    WithValidation(payload): WithValidation<Json<V1UpdateProfilePayload>>,
) -> impl IntoResponse {
    let user_id = extract_user_id_from_headers(headers);
    if let Some(user_id) = user_id {
        let payload = payload.into_inner();
        let updated_user = User::update(
            &state.db_pool,
            user_id,
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

// Placeholder function for extracting user_id from a token
fn extract_user_id_from_headers(headers: TypedHeader<Cookie>) -> Option<i32> {
    let user_id = headers.get("session_id")?;
    user_id.parse::<i32>().ok()
    // Some(1)
}
