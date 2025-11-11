use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use axum_macros::debug_handler;
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde_json::json;
use tower_sessions_redis_store::fred::prelude::*;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{user, user_session},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedQuery,
    services::auth::AuthSession,
    AppState,
};

use super::{
    service::get_google_oauth_client,
    validator::{GoogleCallbackQuery, GoogleExchangeRequest, GoogleUserInfo},
};

#[debug_handler]
#[instrument(skip(state), fields(result))]
pub async fn google_login(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("Initiating Google OAuth login");

    let client = get_google_oauth_client()?;

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    let csrf_key = format!("oauth:csrf:{}", csrf_token.secret());
    state
        .redis_pool
        .set::<(), _, _>(
            &csrf_key,
            csrf_token.secret(),
            Some(fred::types::Expiration::EX(600)),
            None,
            false,
        )
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to store CSRF token");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to store CSRF token")
        })?;

    info!("Generated auth URL with CSRF token");
    tracing::Span::current().record("result", "success");

    Ok(Redirect::temporary(auth_url.as_str()))
}

#[debug_handler]
#[instrument(skip(state, auth, query), fields(user_id, result))]
pub async fn google_callback(
    State(state): State<AppState>,
    mut auth: AuthSession,
    ValidatedQuery(query): ValidatedQuery<GoogleCallbackQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("Processing Google OAuth callback");

    verify_csrf_token(&state, &query.state).await?;

    let client = get_google_oauth_client()?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to exchange authorization code");
            tracing::Span::current().record("result", "token_exchange_failed");
            ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to exchange authorization code")
                .with_details(e.to_string())
        })?;

    let access_token = token_result.access_token().secret();

    let user_info = fetch_google_user_info(access_token).await?;

    info!(google_id = %user_info.id, email = %user_info.email, "Retrieved user info from Google");

    let user = find_or_create_user(&state, user_info).await?;

    tracing::Span::current().record("user_id", user.id);

    auth.login(&user).await.map_err(|e| {
        error!(error = %e, user_id = user.id, "Failed to create session");
        tracing::Span::current().record("result", "session_creation_failed");
        ErrorResponse::new(ErrorCode::InternalServerError).with_message("Failed to create session")
    })?;

    let _ = user_session::Entity::create(
        &state.sea_db,
        user_session::NewUserSession::new(user.id, Some("Google OAuth".to_string()), None),
    )
    .await;

    info!(user_id = user.id, "Google login successful");
    tracing::Span::current().record("result", "success");

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let redirect_url = format!("{}/auth/google/success", frontend_url);

    Ok(Redirect::temporary(&redirect_url))
}

#[debug_handler]
pub async fn google_user_info(auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    match auth.user {
        Some(user) => Ok((StatusCode::OK, Json(json!(user)))),
        None => Err(ErrorResponse::new(ErrorCode::Unauthorized)),
    }
}

/// Exchange authorization code from client-side OAuth callback
/// This endpoint allows the client to receive the OAuth callback and then
/// send the authorization code to the API for token exchange.
///
/// Flow:
/// 1. Client calls GET /auth/google/v1/login to get auth URL
/// 2. Client redirects user to Google OAuth (with client's redirect_uri)
/// 3. Google redirects back to CLIENT with code and state
/// 4. Client POSTs code and state to this endpoint
/// 5. API exchanges code, creates session, returns user info
#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id, result))]
pub async fn google_exchange(
    State(state): State<AppState>,
    mut auth: AuthSession,
    crate::extractors::ValidatedJson(payload): crate::extractors::ValidatedJson<
        GoogleExchangeRequest,
    >,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("Processing Google OAuth code exchange from client");

    verify_csrf_token(&state, &payload.state).await?;

    let client = get_google_oauth_client()?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(payload.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to exchange authorization code");
            tracing::Span::current().record("result", "token_exchange_failed");
            ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to exchange authorization code")
                .with_details(e.to_string())
        })?;

    let access_token = token_result.access_token().secret();

    let user_info = fetch_google_user_info(access_token).await?;

    info!(google_id = %user_info.id, email = %user_info.email, "Retrieved user info from Google");

    let user = find_or_create_user(&state, user_info).await?;

    tracing::Span::current().record("user_id", user.id);

    auth.login(&user).await.map_err(|e| {
        error!(error = %e, user_id = user.id, "Failed to create session");
        tracing::Span::current().record("result", "session_creation_failed");
        ErrorResponse::new(ErrorCode::InternalServerError).with_message("Failed to create session")
    })?;

    let _ = user_session::Entity::create(
        &state.sea_db,
        user_session::NewUserSession::new(user.id, Some("Google OAuth".to_string()), None),
    )
    .await;

    info!(
        user_id = user.id,
        "Google login successful via client exchange"
    );
    tracing::Span::current().record("result", "success");

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "user": user,
            "message": "Successfully authenticated with Google"
        })),
    ))
}

async fn verify_csrf_token(state: &AppState, token: &str) -> Result<(), ErrorResponse> {
    let csrf_key = format!("oauth:csrf:{}", token);
    let stored_token: Option<String> = state.redis_pool.get(&csrf_key).await.map_err(|e| {
        error!(error = ?e, "Failed to retrieve CSRF token");
        ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to verify CSRF token")
    })?;

    match stored_token {
        Some(stored) if stored == token => {
            let _: () = state.redis_pool.del(&csrf_key).await.map_err(|e| {
                error!(error = ?e, "Failed to delete CSRF token");
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message("Failed to delete CSRF token")
            })?;
            Ok(())
        }
        _ => {
            warn!("Invalid or missing CSRF token");
            Err(ErrorResponse::new(ErrorCode::InvalidToken).with_message("Invalid CSRF token"))
        }
    }
}

async fn fetch_google_user_info(access_token: &str) -> Result<GoogleUserInfo, ErrorResponse> {
    let client = reqwest::Client::new();
    client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to fetch user info from Google");
            ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to fetch user info from Google")
        })?
        .json()
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to parse user info from Google");
            ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to parse user info from Google")
        })
}

async fn find_or_create_user(
    state: &AppState,
    user_info: GoogleUserInfo,
) -> Result<user::Model, ErrorResponse> {
    if let Some(existing_user) =
        user::Entity::find_by_google_id(&state.sea_db, user_info.id.clone()).await?
    {
        info!(
            user_id = existing_user.id,
            "Existing user found by Google ID"
        );
        return Ok(existing_user);
    }

    if let Some(mut existing_user) =
        user::Entity::find_by_email(&state.sea_db, user_info.email.clone()).await?
    {
        info!(
            user_id = existing_user.id,
            "Linking Google account to existing user"
        );

        use sea_orm::ActiveModelTrait;
        let mut active: user::ActiveModel = existing_user.clone().into();
        active.google_id = sea_orm::Set(Some(user_info.id.clone()));
        active.oauth_provider = sea_orm::Set(Some("google".to_string()));
        active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());

        existing_user = active.update(&state.sea_db).await.map_err(|e| {
            error!(error = ?e, "Failed to link Google account");
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Failed to link Google account")
        })?;

        return Ok(existing_user);
    }

    info!("Creating new user from Google account");
    user::Entity::create_from_google(
        &state.sea_db,
        user_info.id.clone(),
        user_info.email.clone(),
        user_info.name.clone(),
    )
    .await
}
