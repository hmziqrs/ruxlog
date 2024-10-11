use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::user::{NewUser, User},
    modules::auth_v1::validator::{V1LoginPayload, V1RegisterPayload},
    AppState,
};

#[debug_handler]
pub async fn log_in(
    state: State<AppState>,
    WithValidation(payload): WithValidation<Json<V1LoginPayload>>,
) -> impl IntoResponse {
    let payload = payload.into_inner();
    let user = User::find_by_email(&state.db_pool, payload.email.as_str()).await;

    println!("{:?}", user);

    match user {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": err.to_string(),
                "message": "User not found",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn register(
    state: State<AppState>,
    WithValidation(payload): WithValidation<Json<V1RegisterPayload>>,
) -> impl IntoResponse {
    let payload = payload.into_inner();
    let new_user = NewUser {
        name: payload.name.clone(),
        email: payload.email.clone(),
        password: payload.password.clone(),
    };
    let user = User::create(&state.db_pool, new_user).await;

    match user {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": err.to_string(),
                    "message": "Failed to create user",
                })),
            )
                .into_response()
        }
    }
}
