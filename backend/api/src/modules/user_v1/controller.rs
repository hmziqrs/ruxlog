#[cfg_attr(not(feature = "full"), allow(unused_imports))]
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
#[cfg_attr(not(feature = "full"), allow(unused_imports))]
use crate::{
    db::sea_models::user::{Entity as User, UserRole},
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
            Err(err)
        }
    }
}

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(auth, state, payload))]
pub async fn admin_create(
    auth: AuthSession,
    state: State<AppState>,
    payload: ValidatedJson<V1AdminCreateUserPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // PRIV-ESCAL-1: the route guard only requires ROLE_ADMIN. Enforce here that
    // an admin cannot create a user whose role exceeds their own — otherwise an
    // ADMIN could mint a SUPER_ADMIN, defeating the top tier that admin_acl_v1
    // / seed_v1 gate with ROLE_SUPER_ADMIN.
    let caller_level = auth.user.as_ref().map(|u| u.role.to_i32()).ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    let requested = UserRole::from_str(&payload.0.role)
        .map_err(|_| ErrorResponse::new(ErrorCode::InvalidInput).with_message("Invalid role"))?;
    if requested.to_i32() > caller_level {
        warn!(
            caller_level,
            requested = %payload.0.role,
            "Admin attempted to create a user above their own role"
        );
        return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("You cannot create a user with a role higher than your own"));
    }

    let payload = payload.0.into_new_user();
    let user = User::admin_create(&state.sea_db, &state.object_storage.public_url, payload).await?;
    info!(user_id = user.id, "Admin created user");
    Ok((StatusCode::CREATED, Json(json!(user))))
}

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(auth, state), fields(user_id))]
pub async fn admin_delete(
    auth: AuthSession,
    state: State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // PRIV-ESCAL-1 (admin_delete): the sibling handlers (admin_create /
    // admin_update / admin_change_password) all enforce the role hierarchy, but
    // this one was missed — an ADMIN could delete a SUPER_ADMIN (or any peer
    // ADMIN), destroying a superior's account and — if the last SUPER_ADMIN is
    // removed — making the ROLE_SUPER_ADMIN-only seed/admin-acl routes
    // permanently unreachable. Mirror the caller-vs-target check used by
    // admin_change_password: forbid touching a user at/above the caller's own
    // level, and forbid self-deletion through the admin path.
    let caller = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    let caller_level = caller.role.to_i32();
    if caller.id == user_id {
        return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("You cannot delete your own account via the admin path"));
    }
    if let Some(target) = User::get_by_id(&state.sea_db, user_id).await? {
        if target.role.to_i32() >= caller_level {
            warn!(
                caller_level,
                target_level = target.role.to_i32(),
                "Admin attempted to delete an equal/higher-role user"
            );
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("You cannot delete a user at or above your own role"));
        }
    }
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

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(auth, state, payload), fields(user_id))]
pub async fn admin_update(
    auth: AuthSession,
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<V1AdminUpdateUserPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // PRIV-ESCAL-1: the route guard only requires ROLE_ADMIN, so enforce the
    // role hierarchy here. (a) the requested role must not exceed the caller's
    // own, and (b) the caller may not modify a user already at/above their own
    // level. Together these close ADMIN -> SUPER_ADMIN self-escalation and the
    // silent takeover primitive an admin had by editing/demoting a superior. An
    // admin still manages their own profile via the self-service
    // /user/v1/update endpoint, so blocking equal-rank edits here is safe.
    let caller_level = auth.user.as_ref().map(|u| u.role.to_i32()).ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;

    if let Some(role_str) = payload.0.role.as_deref() {
        let requested = UserRole::from_str(role_str).map_err(|_| {
            ErrorResponse::new(ErrorCode::InvalidInput).with_message("Invalid role")
        })?;
        if requested.to_i32() > caller_level {
            warn!(
                caller_level,
                requested = role_str,
                "Admin attempted to assign a role above their own"
            );
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("You cannot assign a role higher than your own"));
        }
    }

    if let Some(target) = User::get_by_id(&state.sea_db, user_id).await? {
        if target.role.to_i32() >= caller_level {
            warn!(
                caller_level,
                target_level = target.role.to_i32(),
                "Admin attempted to modify an equal/higher-role user"
            );
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("You cannot modify a user at or above your own role"));
        }
    }

    let payload = payload.0.into_update_user();
    match User::admin_update(
        &state.sea_db,
        &state.object_storage.public_url,
        user_id,
        payload,
    )
    .await
    {
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
            Err(err)
        }
    }
}

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(auth, state, payload), fields(user_id))]
pub async fn admin_change_password(
    auth: AuthSession,
    state: State<AppState>,
    Path(user_id): Path<i32>,
    payload: ValidatedJson<AdminChangePassword>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // PRIV-ESCAL-1: forbid resetting the password of a user at/above the
    // caller's own level — otherwise an ADMIN could set a known password on a
    // SUPER_ADMIN account (account takeover of a superior).
    let caller_level = auth.user.as_ref().map(|u| u.role.to_i32()).ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    if let Some(target) = User::get_by_id(&state.sea_db, user_id).await? {
        if target.role.to_i32() >= caller_level {
            warn!(
                caller_level,
                target_level = target.role.to_i32(),
                "Admin attempted to reset password of an equal/higher-role user"
            );
            return Err(
                ErrorResponse::new(ErrorCode::OperationNotAllowed).with_message(
                    "You cannot reset the password of a user at or above your own role",
                ),
            );
        }
    }
    User::change_password(&state.sea_db, user_id, payload.0.password).await?;
    info!(user_id, "Admin changed user password");
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Password changed successfully" })),
    ))
}

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn admin_list(
    state: State<AppState>,
    payload: ValidatedJson<V1AdminUserQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let query = payload.0.into_user_query();
    let page = query.page.unwrap_or(1);

    let (users, total) =
        User::admin_list(&state.sea_db, &state.object_storage.public_url, query).await?;
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

#[cfg(feature = "user-management")]
#[debug_handler]
#[instrument(skip(state), fields(user_id))]
pub async fn admin_view(
    state: State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match User::find_by_id_with_relations(&state.sea_db, &state.object_storage.public_url, user_id)
        .await
    {
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
