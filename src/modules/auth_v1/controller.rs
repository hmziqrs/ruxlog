use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;

use axum_client_ip::ClientIp;
use serde_json::json;

use crate::{
    db::sea_models::{user, user_session},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    modules::auth_v1::validator::{
        V1LoginPayload, V1RegisterPayload, V1TwoFADisablePayload, V1TwoFAVerifyPayload,
    },
    services::auth::{AuthSession, Credentials},
    AppState,
};

#[debug_handler]
pub async fn log_out(mut auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    match auth.logout().await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({"message": "Logged out"})))),
        Err(_) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("An error occurred while logging out")),
    }
}

#[debug_handler]
pub async fn log_in(
    State(state): State<AppState>,
    mut auth: AuthSession,
    ClientIp(secure_ip): ClientIp,
    headers: HeaderMap,
    payload: ValidatedJson<V1LoginPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.authenticate(Credentials::Password(payload.0)).await;

    match user {
        Ok(Some(user)) => match auth.login(&user).await {
            Ok(_) => {
                let ip = Some(secure_ip.to_string());
                let device = headers
                    .get("user-agent")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let _ = user_session::Entity::create(
                    &state.sea_db,
                    user_session::NewUserSession::new(user.id, device, ip),
                )
                .await;
                Ok((StatusCode::OK, Json(json!(user))))
            }
            Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("An error occurred while logging in")
                .with_details(err.to_string())),
        },
        Ok(None) => Err(ErrorResponse::new(ErrorCode::InvalidCredentials)),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn register(
    state: State<AppState>,
    payload: ValidatedJson<V1RegisterPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0;

    match user::Entity::create(&state.sea_db, payload.into_new_user()).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(json!(user)))),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn twofa_setup(
    State(_state): State<AppState>,
    auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();

    // Stub secret generation (hex). Real TOTP should use base32 and persist to DB.
    let secret_bytes: [u8; 20] = rand::random();
    let secret_hex = hex::encode(secret_bytes);
    let issuer = "Ruxlog";
    let label = format!("{}:{}", issuer, user.email);
    let otpauth_url = format!(
        "otpauth://totp/{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        urlencoding::encode(&label),
        secret_hex,
        urlencoding::encode(issuer)
    );

    Ok((
        StatusCode::OK,
        Json(json!({
            "secret": secret_hex,
            "otpauth_url": otpauth_url,
        })),
    ))
}

#[debug_handler]
pub async fn twofa_verify(
    State(_state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1TwoFAVerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();
    let _payload = payload.0;

    Ok((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "message": "2FA verification is not implemented yet" })),
    ))
}

#[debug_handler]
pub async fn twofa_disable(
    State(_state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1TwoFADisablePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();
    let _payload = payload.0;

    Ok((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "message": "2FA disable is not implemented yet" })),
    ))
}

#[debug_handler]
pub async fn sessions_list(
    State(state): State<AppState>,
    auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let page = 1;

    match user_session::Entity::list_by_user(&state.sea_db, user.id, Some(page)).await {
        Ok((sessions, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": sessions,
                "total": total,
                "page": page,
            })),
        )),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn sessions_terminate(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match user_session::Entity::revoke(&state.sea_db, id).await {
        Ok(Some(_session)) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Session terminated" })),
        )),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)),
        Err(err) => Err(err.into()),
    }
}
