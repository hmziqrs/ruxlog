use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;

use axum_client_ip::ClientIp;

use sea_orm::ActiveModelTrait;
use serde_json::json;

use crate::{
    db::sea_models::{user, user_session},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    modules::auth_v1::validator::{
        V1LoginPayload, V1RegisterPayload, V1TwoFADisablePayload, V1TwoFAVerifyPayload,
    },
    services::auth::{AuthSession, Credentials},
    utils::twofa,
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
    State(state): State<AppState>,
    auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();

    // Generate base32 secret and backup codes, persist to user
    let secret_b32 = twofa::generate_secret_base32(20);
    let otpauth_url = twofa::build_otpauth_url(
        &user.email,
        "Ruxlog",
        &secret_b32,
        twofa::DEFAULT_TOTP_DIGITS,
    );

    // Generate and hash backup codes (store hashes only)
    let backup_codes = twofa::generate_backup_codes(10);
    let backup_hashes = twofa::hash_backup_codes(&backup_codes);
    let backup_hashes_json = serde_json::json!(backup_hashes);

    // Persist on user
    let existing = user::Entity::find_by_id_with_404(&state.sea_db, user.id).await?;
    let mut active: user::ActiveModel = existing.into();
    active.two_fa_enabled = sea_orm::Set(false);
    active.two_fa_secret = sea_orm::Set(Some(secret_b32.clone()));
    active.two_fa_backup_codes = sea_orm::Set(Some(backup_hashes_json));
    active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
    active.update(&state.sea_db).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "secret": secret_b32,
            "otpauth_url": otpauth_url,
            "backup_codes": backup_codes,
        })),
    ))
}

#[debug_handler]
pub async fn twofa_verify(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1TwoFAVerifyPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let payload = payload.0;

    let existing = user::Entity::find_by_id_with_404(&state.sea_db, user.id).await?;
    let secret = match &existing.two_fa_secret {
        Some(s) => s.clone(),
        None => {
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("2FA not initialized"))
        }
    };

    // If code matches TOTP, enable 2FA. Otherwise, try backup code consumption.
    let totp_ok = twofa::verify_totp_code_now(&secret, &payload.code);

    if totp_ok {
        let mut active: user::ActiveModel = existing.into();
        active.two_fa_enabled = sea_orm::Set(true);
        active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
        let updated = active.update(&state.sea_db).await?;
        return Ok((StatusCode::OK, Json(json!(updated))));
    }

    // Try backup code if provided
    if let Some(backup_code) = payload.backup_code {
        if let Some(stored) = &existing.two_fa_backup_codes {
            let stored_vec: Vec<String> =
                serde_json::from_value(stored.clone()).unwrap_or_else(|_| vec![]);
            if let Some(updated_hashes) = twofa::consume_backup_code(&stored_vec, &backup_code) {
                let mut active: user::ActiveModel = existing.into();
                active.two_fa_enabled = sea_orm::Set(true);
                active.two_fa_backup_codes = sea_orm::Set(Some(serde_json::json!(updated_hashes)));
                active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
                let updated = active.update(&state.sea_db).await?;
                return Ok((StatusCode::OK, Json(json!(updated))));
            }
        }
    }

    Err(ErrorResponse::new(ErrorCode::InvalidToken).with_message("Invalid 2FA code"))
}

#[debug_handler]
pub async fn twofa_disable(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1TwoFADisablePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let payload = payload.0;

    let existing = user::Entity::find_by_id_with_404(&state.sea_db, user.id).await?;

    // If 2FA is enabled and a code is provided, verify it; allow disable with valid code or backup code
    if existing.two_fa_enabled {
        if let Some(code) = payload.code.clone() {
            let secret = existing.two_fa_secret.clone().unwrap_or_default();
            let totp_ok = if secret.is_empty() {
                false
            } else {
                twofa::verify_totp_code_now(&secret, &code)
            };

            let mut backup_ok = false;
            if !totp_ok {
                if let Some(stored) = &existing.two_fa_backup_codes {
                    let stored_vec: Vec<String> =
                        serde_json::from_value(stored.clone()).unwrap_or_else(|_| vec![]);
                    backup_ok = twofa::consume_backup_code(&stored_vec, &code).is_some();
                }
            }

            if !totp_ok && !backup_ok {
                return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                    .with_message("Invalid 2FA or backup code"));
            }
        } else {
            // Require a code if 2FA is enabled
            return Err(ErrorResponse::new(ErrorCode::MissingRequiredField)
                .with_message("code is required"));
        }
    }

    // Disable and clear secrets
    let mut active: user::ActiveModel = existing.into();
    active.two_fa_enabled = sea_orm::Set(false);
    active.two_fa_secret = sea_orm::Set(None);
    active.two_fa_backup_codes = sea_orm::Set(None);
    active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());
    let updated = active.update(&state.sea_db).await?;

    Ok((StatusCode::OK, Json(json!(updated))))
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
