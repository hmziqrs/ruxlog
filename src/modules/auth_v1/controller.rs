use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use crate::{
    db::models::user::User,
    error::{ErrorCode, ErrorResponse},
    modules::auth_v1::validator::{V1LoginPayload, V1RegisterPayload},
    services::auth::{AuthSession, Credentials},
    AppState,
};

#[debug_handler]
pub async fn log_out(mut auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    match auth.logout().await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({"message": "Logged out"})))),
        Err(_) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("An error occurred while logging out"))
    }
}

#[debug_handler]
pub async fn log_in(
    _state: State<AppState>,
    mut auth: AuthSession,
    payload: Valid<Json<V1LoginPayload>>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.into_inner().0;
    let user = auth.authenticate(Credentials::Password(payload)).await;

    match user {
        Ok(Some(user)) => match auth.login(&user).await {
            Ok(_) => Ok((StatusCode::OK, Json(json!(user)))),
            Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("An error occurred while logging in")
                .with_details(err.to_string()))
        },
        Ok(None) => {
            // No user found or password mismatch - return InvalidCredentials
            Err(ErrorResponse::new(ErrorCode::InvalidCredentials))
        },
        Err(err) => {
            // Convert the AuthError to our standard ErrorResponse
            Err(err.into())
        }
    }
}

#[debug_handler]
pub async fn register(
    state: State<AppState>,
    payload: Valid<Json<V1RegisterPayload>>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.into_inner().0.into_new_user();

    match User::create(&state.db_pool, payload).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(json!(user)))),
        Err(err) => Err(ErrorResponse::new(ErrorCode::DuplicateEntry)
            .with_message("Failed to create user")
            .with_details(err.to_string()))
    }
}
