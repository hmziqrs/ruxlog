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
    db::sea_models::user::Entity as User,
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

#[debug_handler]
pub async fn get_profile(auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    match auth.user {
        Some(user) => Ok((StatusCode::OK, Json(json!(user)))),
        None => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("No user with this ID exists")),
    }
}

#[debug_handler]
pub async fn update_profile(
    auth: AuthSession,
    state: State<AppState>,
    payload: ValidatedJson<V1UpdateProfilePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("You must be logged in to access this resource")
    })?;

    let payload = payload.0.into_update_user();
    match User::update(&state.sea_db, user.id, payload).await {
        Ok(Some(user)) => Ok((StatusCode::OK, Json(json!(user)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("User could not be found or updated")),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_create(
    state: State<AppState>,
    payload: ValidatedJson<V1AdminCreateUserPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0.into_new_user();
    let user = User::admin_create(&state.sea_db, payload).await?;
    Ok((StatusCode::CREATED, Json(json!(user))))
}

#[debug_handler]
pub async fn admin_delete(
    state: State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match User::admin_delete(&state.sea_db, user_id).await {
        Ok(1) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "User deleted successfully" })),
        )),
        Ok(0) => {
            Err(ErrorResponse::new(ErrorCode::RecordNotFound).with_message("User does not exist"))
        }
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "User deleted successfully" })),
        )),
        Err(err) => Err(err),
    }
}

#[debug_handler]
pub async fn admin_update(
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<V1AdminUpdateUserPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0.into_update_user();
    match User::admin_update(&state.sea_db, user_id, payload).await {
        Ok(Some(user)) => Ok((StatusCode::OK, Json(json!(user)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("No user with this ID exists")),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_change_password(
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<AdminChangePassword>,
) -> Result<impl IntoResponse, ErrorResponse> {
    User::change_password(&state.sea_db, user_id, payload.0.password).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Password changed successfully" })),
    ))
}

#[debug_handler]
pub async fn admin_list(
    state: State<AppState>,
    payload: ValidatedJson<V1AdminUserQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let query = payload.0.into_user_query();
    let page = query.page.unwrap_or(1);

    let (users, total) = User::admin_list(&state.sea_db, query).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "data": users,
            "total": total,
            "per_page": User::PER_PAGE,
            "page": page,
        })),
    ))
}

#[debug_handler]
pub async fn admin_view(
    state: State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match User::get_by_id(&state.sea_db, user_id).await {
        Ok(Some(user)) => Ok((StatusCode::OK, Json(json!(user)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("No user with this ID exists")),
        Err(err) => Err(err.into()),
    }
}
