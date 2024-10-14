use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::user::{NewUser, User},
    modules::auth_v1::validator::{V1LoginPayload, V1RegisterPayload},
    services::auth::{AuthSession, Credentials},
    AppState,
};

#[debug_handler]
pub async fn log_out() -> impl IntoResponse {
    // Clear the session cookie
    (
        StatusCode::OK,
        Json(json!({
            "message": "Logged out successfully",
        })),
    )
        .into_response()

    // response.headers_mut().append(
    //     axum::http::header::SET_COOKIE,
    //     axum::http::HeaderValue::from_str("session_id=; Path=/; Max-Age=0; HttpOnly").unwrap(),
    // );
}

#[debug_handler]
pub async fn log_in(
    state: State<AppState>,
    mut auth: AuthSession,
    WithValidation(payload): WithValidation<Json<V1LoginPayload>>,
) -> impl IntoResponse {
    let payload = payload.into_inner();
    let user = auth
        .authenticate(Credentials::Password(payload.clone()))
        .await;
    println!("User: {:?}", user);

    match user {
        Ok(Some(user)) => {
            // Set the session ID in a cookie
            (StatusCode::OK, Json(json!(user))).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found",
                "message": "No user with this email exists",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "An error occurred while fetching the user",
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
        Ok(user) => (StatusCode::CREATED, Json(json!(user))),
        Err(err) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to create user",
            })),
        ),
    }
}
