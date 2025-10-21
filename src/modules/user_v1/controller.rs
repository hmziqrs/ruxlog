use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;

use super::validator::*;
use crate::{
    db::sea_models::user::Entity as User, extractors::ValidatedJson, services::auth::AuthSession,
    AppState,
};

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
    payload: ValidatedJson<V1UpdateProfilePayload>,
) -> impl IntoResponse {
    if let Some(user) = auth.user {
        let payload = payload.0.into_update_user();
        let updated_user = User::update(&state.sea_db, user.id, payload).await;
        match updated_user {
            Ok(Some(user)) => {
                return (StatusCode::OK, Json(json!(user))).into_response();
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "User not found",
                        "message": "User could not be found or updated",
                    })),
                )
                    .into_response();
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
    state: State<AppState>,
    payload: ValidatedJson<V1AdminCreateUserPayload>,
) -> impl IntoResponse {
    let payload = payload.0.into_new_user();
    let conn = &state.sea_db;

    match User::admin_create(conn, payload).await {
        Ok(user) => (StatusCode::CREATED, Json(json!(user))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_delete(state: State<AppState>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let conn = &state.sea_db;
    match User::admin_delete(conn, user_id).await {
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
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<V1AdminUpdateUserPayload>,
) -> impl IntoResponse {
    let payload = payload.0.into_update_user();
    let conn = &state.sea_db;

    match User::admin_update(conn, user_id, payload).await {
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
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<AdminChangePassword>,
) -> impl IntoResponse {
    let conn = &state.sea_db;

    match User::change_password(conn, user_id, payload.0.password).await {
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
    state: State<AppState>,
    payload: ValidatedJson<V1AdminUserQueryParams>,
) -> impl IntoResponse {
    let query = payload.0.into_user_query();
    let conn = &state.sea_db;

    match User::admin_list(conn, query).await {
        Ok(users_with_count) => (StatusCode::OK, Json(json!(users_with_count))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn admin_view(state: State<AppState>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let conn = &state.sea_db;
    match User::get_by_id(conn, user_id).await {
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
