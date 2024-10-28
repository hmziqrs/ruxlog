use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use super::validator::*;
use crate::{db::models::user::User, services::auth::AuthSession, AppState};

#[debug_handler]
pub async fn get_profile(auth: AuthSession) -> impl IntoResponse {
    match auth.user {
        Some(user) => (StatusCode::OK, Json(user)).into_response(),
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
    payload: Valid<Json<V1UpdateProfilePayload>>,
) -> impl IntoResponse {
    if let Some(user) = auth.user {
        let payload = payload.into_inner().0.into_update_user();
        let updated_user = User::update(&state.db_pool, user.id, payload).await;
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

#[debug_handler]
pub async fn admin_create(
    State(state): State<AppState>,
    payload: Valid<Json<AdminCreateUser>>,
) -> impl IntoResponse {
    let payload = payload.into_inner().0.into_new_user();

    match User::admin_create(&state.db_pool, payload).await {
        Ok(user) => (StatusCode::CREATED, Json(json!(user))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_delete(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    match User::admin_delete(&state.db_pool, user_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_update(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    payload: Valid<Json<AdminUpdateUser>>,
) -> impl IntoResponse {
    let payload = payload.into_inner().0.into_update_user();

    match User::admin_update(&state.db_pool, user_id, payload).await {
        Ok(Some(user)) => (StatusCode::OK, Json(json!(user))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "message": "No user with this ID exists",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_change_password(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    payload: Valid<Json<AdminChangePassword>>,
) -> impl IntoResponse {
    let payload = payload.into_inner().0;

    match User::admin_change_password(&state.db_pool, user_id, payload.password).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_list(
    State(state): State<AppState>,
    payload: Valid<Json<V1AdminUserQueryParams>>,
) -> impl IntoResponse {
    let query = payload.into_inner().0.into_user_query();

    match User::admin_list(&state.db_pool, query).await {
        Ok(users) => (StatusCode::OK, Json(json!(users))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_view(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    match User::admin_view(&state.db_pool, user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(json!(user))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "message": "No user with this ID exists",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}
