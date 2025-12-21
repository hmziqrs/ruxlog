//! Google OAuth provider implementation

use async_trait::async_trait;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, EmptyExtraTokenFields, RedirectUrl,
    StandardTokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

use super::provider::{OAuthProvider, OAuthProviderConfig, OAuthUserInfo};
use crate::error::{AuthError, AuthErrorCode};

/// Google OAuth user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    /// Google user ID
    pub id: String,
    /// User's email address
    pub email: Option<String>,
    /// Whether the email is verified
    #[serde(default)]
    pub verified_email: bool,
    /// User's display name
    pub name: Option<String>,
    /// Given (first) name
    pub given_name: Option<String>,
    /// Family (last) name
    pub family_name: Option<String>,
    /// Profile picture URL
    pub picture: Option<String>,
    /// User's locale
    pub locale: Option<String>,
}

impl OAuthUserInfo for GoogleUserInfo {
    fn provider_user_id(&self) -> &str {
        &self.id
    }

    fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn avatar_url(&self) -> Option<&str> {
        self.picture.as_deref()
    }

    fn email_verified(&self) -> bool {
        self.verified_email
    }
}

/// Google OAuth provider
#[derive(Clone)]
pub struct GoogleProvider {
    client: BasicClient,
    config: OAuthProviderConfig,
    http_client: reqwest::Client,
}

impl GoogleProvider {
    /// Google authorization endpoint
    const AUTH_URL: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
    /// Google token endpoint
    const TOKEN_URL: &'static str = "https://oauth2.googleapis.com/token";
    /// Google user info endpoint
    const USER_INFO_URL: &'static str = "https://www.googleapis.com/oauth2/v2/userinfo";

    /// Create a new Google OAuth provider
    ///
    /// # Arguments
    /// * `client_id` - Google OAuth client ID
    /// * `client_secret` - Google OAuth client secret
    /// * `redirect_uri` - Redirect URI after OAuth flow
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self, AuthError> {
        let client_id = client_id.into();
        let client_secret = client_secret.into();
        let redirect_uri = redirect_uri.into();

        let auth_url = AuthUrl::new(Self::AUTH_URL.to_string()).map_err(|e| {
            AuthError::new(AuthErrorCode::InternalError).with_message(e.to_string())
        })?;
        let token_url = TokenUrl::new(Self::TOKEN_URL.to_string()).map_err(|e| {
            AuthError::new(AuthErrorCode::InternalError).with_message(e.to_string())
        })?;

        let client = BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri.clone()).map_err(|e| {
            AuthError::new(AuthErrorCode::InternalError).with_message(e.to_string())
        })?);

        let config = OAuthProviderConfig {
            client_id,
            client_secret,
            redirect_uri,
            auth_url: Self::AUTH_URL.to_string(),
            token_url: Self::TOKEN_URL.to_string(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            user_info_url: Self::USER_INFO_URL.to_string(),
        };

        Ok(Self {
            client,
            config,
            http_client: reqwest::Client::new(),
        })
    }

    /// Create from environment variables
    ///
    /// Reads:
    /// - `GOOGLE_CLIENT_ID`
    /// - `GOOGLE_CLIENT_SECRET`
    /// - `GOOGLE_REDIRECT_URI`
    pub fn from_env() -> Result<Self, AuthError> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|_| {
            AuthError::new(AuthErrorCode::InternalError)
                .with_message("GOOGLE_CLIENT_ID not set")
        })?;

        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| {
            AuthError::new(AuthErrorCode::InternalError)
                .with_message("GOOGLE_CLIENT_SECRET not set")
        })?;

        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").map_err(|_| {
            AuthError::new(AuthErrorCode::InternalError)
                .with_message("GOOGLE_REDIRECT_URI not set")
        })?;

        Self::new(client_id, client_secret, redirect_uri)
    }
}

#[async_trait]
impl OAuthProvider for GoogleProvider {
    const PROVIDER_ID: &'static str = "google";

    type UserInfo = GoogleUserInfo;

    fn client(&self) -> &BasicClient {
        &self.client
    }

    fn config(&self) -> &OAuthProviderConfig {
        &self.config
    }

    async fn exchange_code(
        &self,
        code: AuthorizationCode,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, AuthError> {
        use oauth2::reqwest::async_http_client;

        self.client
            .exchange_code(code)
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to exchange Google authorization code");
                AuthError::new(AuthErrorCode::OAuthError)
                    .with_message("Failed to exchange authorization code")
            })
    }

    async fn fetch_user_info(&self, access_token: &str) -> Result<GoogleUserInfo, AuthError> {
        self.http_client
            .get(&self.config.user_info_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to fetch Google user info");
                AuthError::new(AuthErrorCode::OAuthError)
                    .with_message("Failed to fetch user info from Google")
            })?
            .json()
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to parse Google user info");
                AuthError::new(AuthErrorCode::OAuthError)
                    .with_message("Failed to parse user info from Google")
            })
    }
}
