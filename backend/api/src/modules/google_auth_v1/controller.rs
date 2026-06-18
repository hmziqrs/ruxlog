use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use axum_macros::debug_handler;
use oauth2::{
    reqwest::async_http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    Scope, TokenResponse,
};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;
use tower_sessions_redis_store::fred::prelude::*;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{user, user_session},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    extractors::ValidatedQuery,
    services::auth::AuthSession,
    AppState,
};

use super::{
    service::{get_google_oauth_client, verify_google_id_token, GoogleIdTokenClaims},
    validator::{GoogleCallbackQuery, GoogleExchangeRequest, GoogleUserInfo},
};

#[debug_handler]
#[instrument(skip(state, session), fields(result))]
pub async fn google_login(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("Initiating Google OAuth login");

    let client = get_google_oauth_client()?;

    // PKCE: protect the authorization-code exchange against code interception /
    // replay. The verifier is stored server-side and consumed at the callback.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Bind the CSRF state to THIS browser session so a state issued to one
    // session cannot complete the flow in another (login CSRF / state replay).
    // The stored value is the PKCE verifier, which makes the lookup non-vacuous
    // (key != value) and lets us recover the verifier at the callback.
    let session_id = oauth_session_id(&session)?;
    store_oauth_state(&state, &session_id, csrf_token.secret(), pkce_verifier.secret()).await?;

    info!("Generated auth URL with PKCE + session-bound CSRF state");
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

    let session_id = oauth_session_id(auth.session())?;
    let pkce_verifier = consume_oauth_state(&state, &session_id, &query.state).await?;

    let client = get_google_oauth_client()?;
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to exchange authorization code");
            tracing::Span::current().record("result", "token_exchange_failed");
            ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to exchange authorization code")
                .with_details(e.to_string())
        })?;

    let user = finish_google_login(&state, &mut auth, token_result).await?;

    info!(user_id = user.id, "Google login successful");
    tracing::Span::current().record("result", "success");

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let redirect_url = format!("{}/auth/google/success", frontend_url);

    Ok(Redirect::temporary(&redirect_url))
}

#[debug_handler(state = AppState)]
pub async fn google_user_info(auth: AuthSession) -> Result<impl IntoResponse, ErrorResponse> {
    match auth.user {
        Some(user) => Ok((StatusCode::OK, Json(json!(user)))),
        None => Err(ErrorResponse::new(ErrorCode::Unauthorized)),
    }
}

/// Exchange authorization code from client-side OAuth callback
///
/// Flow:
/// 1. Client calls GET /auth/google/v1/login to get auth URL
/// 2. Client redirects user to Google OAuth (with client's redirect_uri)
/// 3. Google redirects back to CLIENT with code and state
/// 4. Client POSTs code and state to this endpoint
/// 5. API exchanges code (with PKCE verifier), verifies the id_token, creates
///    session, returns user info
#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id, result))]
pub async fn google_exchange(
    State(state): State<AppState>,
    mut auth: AuthSession,
    ValidatedJson(payload): ValidatedJson<GoogleExchangeRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    info!("Processing Google OAuth code exchange from client");

    let session_id = oauth_session_id(auth.session())?;
    let pkce_verifier = consume_oauth_state(&state, &session_id, &payload.state).await?;

    let client = get_google_oauth_client()?;
    let token_result = client
        .exchange_code(AuthorizationCode::new(payload.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to exchange authorization code");
            tracing::Span::current().record("result", "token_exchange_failed");
            ErrorResponse::new(ErrorCode::ExternalServiceError)
                .with_message("Failed to exchange authorization code")
                .with_details(e.to_string())
        })?;

    let user = finish_google_login(&state, &mut auth, token_result).await?;

    info!(user_id = user.id, "Google login successful via client exchange");
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

/// Shared post-exchange logic: verify the id_token signature/claims (defense in
/// depth), fetch profile data via userinfo, cross-check that the cryptographically
/// verified identity matches the userinfo, then create/link the user + session.
async fn finish_google_login(
    state: &AppState,
    auth: &mut AuthSession,
    token_result: oauth2::StandardTokenResponse<
        super::service::IdTokenFields,
        oauth2::basic::BasicTokenType,
    >,
) -> Result<user::Model, ErrorResponse> {
    let access_token = token_result.access_token().secret();

    // Verify the id_token signature against Google's JWKS when present. This is
    // defense-in-depth: the access token already came from Google's token
    // endpoint for our PKCE-bound code. We additionally require the id_token's
    // `sub`/`email` to match the userinfo response so a token-substitution
    // attack can't pin a verified identity to a different profile.
    // The openid scope is requested, so Google always returns an id_token.
    // Requiring it (and verifying its signature) closes the defense-in-depth
    // gap of trusting only the bearer-authenticated userinfo endpoint.
    let id_token = token_result
        .extra_fields()
        .id_token
        .as_deref()
        .ok_or_else(|| {
            warn!("Google token response omitted id_token; rejecting login");
            tracing::Span::current().record("result", "missing_id_token");
            ErrorResponse::new(ErrorCode::InvalidToken)
                .with_message("OAuth identity verification failed")
        })?;

    let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|_| {
        ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("GOOGLE_CLIENT_ID not configured")
    })?;
    let id_claims: Option<GoogleIdTokenClaims> =
        match verify_google_id_token(id_token, &client_id).await {
            Ok(claims) => Some(claims),
            Err(err) => {
                warn!(error = ?err, "id_token verification failed; rejecting login");
                return Err(err);
            }
        };

    let user_info = fetch_google_user_info(access_token).await?;
    info!(google_id = %user_info.id, email = %user_info.email, "Retrieved user info from Google");

    // Cross-check the verified id_token identity against the userinfo payload.
    if let Some(claims) = &id_claims {
        if claims.sub != user_info.id || claims.email != user_info.email {
            warn!(
                id_sub = %claims.sub,
                userinfo_id = %user_info.id,
                "id_token/userinfo identity mismatch — rejecting login"
            );
            return Err(ErrorResponse::new(ErrorCode::InvalidToken)
                .with_message("OAuth identity verification failed"));
        }
    }

    let user = find_or_create_user(state, user_info).await?;
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

    Ok(user)
}

/// Extract the caller's tower-sessions id, required to bind the OAuth state.
/// Fail closed: without a session we cannot bind state, so we refuse to proceed.
fn oauth_session_id(session: &Session) -> Result<String, ErrorResponse> {
    session
        .id()
        .map(|id| id.to_string())
        .ok_or_else(|| {
            warn!("Google OAuth attempted without a session id");
            ErrorResponse::new(ErrorCode::Unauthorized).with_message("No active session")
        })
}

/// Persist the (session-bound) state → PKCE verifier mapping, single-use, 10 min.
async fn store_oauth_state(
    state: &AppState,
    session_id: &str,
    state_secret: &str,
    pkce_verifier_secret: &str,
) -> Result<(), ErrorResponse> {
    let key = format!("oauth:state:{}:{}", session_id, state_secret);
    let payload = json!({ "v": pkce_verifier_secret }).to_string();
    state
        .redis_pool
        .set::<(), _, _>(
            &key,
            payload,
            Some(fred::types::Expiration::EX(600)),
            None,
            false,
        )
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to store OAuth state");
            ErrorResponse::new(ErrorCode::InternalServerError).with_message("Failed to store OAuth state")
        })
}

#[derive(Deserialize)]
struct StoredState {
    v: String,
}

/// Look up and DELETE the session-bound state, returning the PKCE verifier.
/// Fails closed if the state is missing, expired, or belongs to another session.
async fn consume_oauth_state(
    state: &AppState,
    session_id: &str,
    state_secret: &str,
) -> Result<PkceCodeVerifier, ErrorResponse> {
    let key = format!("oauth:state:{}:{}", session_id, state_secret);

    let stored: Option<String> = state.redis_pool.get(&key).await.map_err(|e| {
        error!(error = ?e, "Failed to retrieve OAuth state");
        ErrorResponse::new(ErrorCode::InternalServerError).with_message("Failed to verify OAuth state")
    })?;

    // Delete first so the state is single-use even if parsing/verification below
    // were to fail — a replayed state must never succeed.
    let _: () = state.redis_pool.del(&key).await.unwrap_or(());

    let stored = stored.ok_or_else(|| {
        warn!("Invalid, expired, or session-mismatched OAuth state");
        ErrorResponse::new(ErrorCode::InvalidToken).with_message("Invalid OAuth state")
    })?;

    let parsed: StoredState = serde_json::from_str(&stored).map_err(|e| {
        error!(error = ?e, "Failed to parse stored OAuth state");
        ErrorResponse::new(ErrorCode::InternalServerError).with_message("Corrupt OAuth state")
    })?;

    Ok(PkceCodeVerifier::new(parsed.v))
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
        info!(user_id = existing_user.id, "Existing user found by Google ID");
        return Ok(existing_user);
    }

    if let Some(existing_user) =
        user::Entity::find_by_email(&state.sea_db, user_info.email.clone()).await?
    {
        // Linking a Google identity onto an existing local account is only safe
        // when the IdP has verified the account actually owns that email.
        // Otherwise an attacker controlling an unverified-at-IdP Google account
        // with the victim's email would take over the account. Fail closed: do
        // not link and do not create a duplicate (the email is already taken).
        if !user_info.verified_email {
            warn!(
                user_id = existing_user.id,
                email = %user_info.email,
                "Refusing to link Google account: IdP email is not verified"
            );
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("Unable to link this Google account"));
        }

        info!(user_id = existing_user.id, "Linking Google account to existing user");

        use sea_orm::ActiveModelTrait;
        let mut active: user::ActiveModel = existing_user.clone().into();
        active.google_id = sea_orm::Set(Some(user_info.id.clone()));
        active.oauth_provider = sea_orm::Set(Some("google".to_string()));
        active.updated_at = sea_orm::Set(chrono::Utc::now().fixed_offset());

        let existing_user = active.update(&state.sea_db).await.map_err(|e| {
            error!(error = ?e, "Failed to link Google account");
            ErrorResponse::new(ErrorCode::InternalServerError).with_message("Failed to link Google account")
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
