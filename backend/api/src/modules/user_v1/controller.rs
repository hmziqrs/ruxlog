use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info, instrument, warn};

use super::validator::*;
use crate::{
    db::sea_models::user::Entity as User,
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

#[debug_handler(state = AppState)]
#[instrument(skip(auth), fields(user_id = auth.user.as_ref().map(|u| u.id)))]
pub async fn get_profile(auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    match auth.user {
        Some(user) => {
            info!(user_id = user.id, "Profile retrieved");
            Ok((StatusCode::OK, Json(json!(user))))
        }
        None => {
            warn!("Profile request with no authenticated user");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("No user with this ID exists"))
        }
    }
}

#[debug_handler]
#[instrument(skip(auth, state, payload), fields(user_id = auth.user.as_ref().map(|u| u.id)))]
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
        Ok(Some(user)) => {
            info!(user_id = user.id, "Profile updated");
            Ok((StatusCode::OK, Json(json!(user))))
        }
        Ok(None) => {
            warn!(user_id = user.id, "User not found during update");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("User could not be found or updated"))
        }
        Err(err) => {
            error!(user_id = user.id, "Failed to update profile: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn admin_create(
    state: State<AppState>,
    payload: ValidatedJson<V1AdminCreateUserPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0.into_new_user();
    let user = User::admin_create(&state.sea_db, payload).await?;
    info!(user_id = user.id, "Admin created user");
    Ok((StatusCode::CREATED, Json(json!(user))))
}

#[debug_handler]
#[instrument(skip(state), fields(user_id))]
pub async fn admin_delete(
    state: State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match User::admin_delete(&state.sea_db, user_id).await {
        Ok(1) => {
            info!(user_id, "Admin deleted user");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "User deleted successfully" })),
            ))
        }
        Ok(0) => {
            warn!(user_id, "Admin tried to delete non-existent user");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound).with_message("User does not exist"))
        }
        Ok(_) => {
            info!(user_id, "Admin deleted user");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "User deleted successfully" })),
            ))
        }
        Err(err) => {
            error!(user_id, "Failed to delete user: {}", err);
            Err(err)
        }
    }
}

#[debug_handler]
#[instrument(skip(state, payload), fields(user_id))]
pub async fn admin_update(
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<V1AdminUpdateUserPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0.into_update_user();
    match User::admin_update(&state.sea_db, user_id, payload).await {
        Ok(Some(user)) => {
            info!(user_id, "Admin updated user");
            Ok((StatusCode::OK, Json(json!(user))))
        }
        Ok(None) => {
            warn!(user_id, "Admin tried to update non-existent user");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("No user with this ID exists"))
        }
        Err(err) => {
            error!(user_id, "Admin failed to update user: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, payload), fields(user_id))]
pub async fn admin_change_password(
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<AdminChangePassword>,
) -> Result<impl IntoResponse, ErrorResponse> {
    User::change_password(&state.sea_db, user_id, payload.0.password).await?;
    info!(user_id, "Admin changed user password");
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Password changed successfully" })),
    ))
}

#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn admin_list(
    state: State<AppState>,
    payload: ValidatedJson<V1AdminUserQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let query = payload.0.into_user_query();
    let page = query.page.unwrap_or(1);

    let (users, total) = User::admin_list(&state.sea_db, query).await?;
    info!(total, page, "Admin listed users");
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
#[instrument(skip(state), fields(user_id))]
pub async fn admin_view(
    state: State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match User::find_by_id_with_relations(&state.sea_db, user_id).await {
        Ok(user) => {
            info!(user_id, "Admin viewed user");
            Ok((StatusCode::OK, Json(json!(user))))
        }
        Err(err) => {
            error!(user_id, "Admin failed to view user: {}", err);
            Err(err)
        }
    }
}
