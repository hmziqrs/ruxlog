//! OAuth provider trait definitions

use async_trait::async_trait;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthorizationCode, CsrfToken, EmptyExtraTokenFields, Scope, StandardTokenResponse,
};
use serde::de::DeserializeOwned;

use crate::error::AuthError;

/// User information retrieved from an OAuth provider
pub trait OAuthUserInfo: Clone + Send + Sync + 'static {
    /// The provider-specific unique user ID
    fn provider_user_id(&self) -> &str;

    /// Email address (if available)
    fn email(&self) -> Option<&str>;

    /// Display name (if available)
    fn name(&self) -> Option<&str>;

    /// Avatar/profile picture URL (if available)
    fn avatar_url(&self) -> Option<&str>;

    /// Whether the email is verified by the provider
    fn email_verified(&self) -> bool;
}

/// Configuration for an OAuth provider
#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret
    pub client_secret: String,
    /// Redirect URI after OAuth flow
    pub redirect_uri: String,
    /// Authorization endpoint URL
    pub auth_url: String,
    /// Token exchange endpoint URL
    pub token_url: String,
    /// Scopes to request
    pub scopes: Vec<String>,
    /// User info endpoint URL
    pub user_info_url: String,
}

/// OAuth provider trait
///
/// Implement this for each OAuth provider (Google, GitHub, Discord, etc.)
#[async_trait]
pub trait OAuthProvider: Clone + Send + Sync + 'static {
    /// Provider identifier (e.g., "google", "github", "discord")
    const PROVIDER_ID: &'static str;

    /// The user info type returned by this provider
    type UserInfo: OAuthUserInfo + DeserializeOwned;

    /// Get the OAuth2 client
    fn client(&self) -> &BasicClient;

    /// Get the provider configuration
    fn config(&self) -> &OAuthProviderConfig;

    /// Generate the authorization URL with a CSRF token
    ///
    /// Returns (authorization_url, csrf_token)
    fn authorization_url(&self) -> (String, CsrfToken) {
        let mut auth = self.client().authorize_url(CsrfToken::new_random);

        for scope in &self.config().scopes {
            auth = auth.add_scope(Scope::new(scope.clone()));
        }

        let (url, csrf) = auth.url();
        (url.to_string(), csrf)
    }

    /// Exchange an authorization code for access tokens
    async fn exchange_code(
        &self,
        code: AuthorizationCode,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, AuthError>;

    /// Fetch user information using an access token
    async fn fetch_user_info(&self, access_token: &str) -> Result<Self::UserInfo, AuthError>;
}

/// Handler for OAuth user creation/linking
///
/// Implement this to connect OAuth authentication to your user system.
#[async_trait]
pub trait OAuthUserHandler<U: Send>: Clone + Send + Sync + 'static {
    /// Find an existing user by OAuth provider ID
    async fn find_by_oauth_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<U>, AuthError>;

    /// Find an existing user by email (for linking accounts)
    async fn find_by_email(&self, email: &str) -> Result<Option<U>, AuthError>;

    /// Link an OAuth account to an existing user
    async fn link_oauth_account(
        &self,
        user: &U,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<U, AuthError>;

    /// Create a new user from OAuth information
    async fn create_from_oauth<I: OAuthUserInfo + Send>(
        &self,
        provider: &str,
        user_info: &I,
    ) -> Result<U, AuthError>;

    /// Find or create a user from OAuth info
    ///
    /// Default implementation:
    /// 1. Try to find by OAuth ID
    /// 2. If not found, try to find by email and link
    /// 3. If not found, create new user
    async fn find_or_create<I: OAuthUserInfo + Send>(
        &self,
        provider: &str,
        user_info: &I,
    ) -> Result<U, AuthError> {
        // 1. Try to find by OAuth ID
        if let Some(user) = self.find_by_oauth_id(provider, user_info.provider_user_id()).await? {
            return Ok(user);
        }

        // 2. Try to find by email and link
        if let Some(email) = user_info.email() {
            if let Some(user) = self.find_by_email(email).await? {
                return self
                    .link_oauth_account(&user, provider, user_info.provider_user_id())
                    .await;
            }
        }

        // 3. Create new user
        self.create_from_oauth(provider, user_info).await
    }
}
