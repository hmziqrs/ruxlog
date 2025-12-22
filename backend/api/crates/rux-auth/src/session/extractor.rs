//! AuthSession extractor for Axum handlers

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use tower_sessions::Session;

use super::state::AuthSessionState;
use crate::error::{AuthError, AuthErrorCode};
use crate::traits::{AuthBackend, AuthUser};

/// Session key for storing auth state
const SESSION_KEY: &str = "rux_auth";

/// The main authentication session extractor
///
/// Use this in your handlers to access the authenticated user and session state.
///
/// ```ignore
/// async fn handler(auth: AuthSession<MyBackend>) -> impl IntoResponse {
///     if let Some(user) = &auth.user {
///         // User is authenticated
///     }
/// }
/// ```
pub struct AuthSession<B: AuthBackend> {
    /// The authenticated user (None if not logged in)
    pub user: Option<B::User>,

    /// The session state (None if not logged in)
    pub state: Option<AuthSessionState<<B::User as AuthUser>::Id>>,

    /// The underlying tower-sessions session
    session: Session,

    /// The auth backend for database operations
    backend: B,
}

impl<B: AuthBackend> AuthSession<B> {
    /// Create a new AuthSession from a backend and session
    ///
    /// This is useful when constructing AuthSession outside of the extractor
    /// (e.g., in middleware that extracts State and Session separately).
    pub async fn new(backend: B, session: Session) -> Self {
        // Try to load auth state from session
        let auth_state: Option<AuthSessionState<<B::User as AuthUser>::Id>> =
            session.get(SESSION_KEY).await.ok().flatten();

        // If we have auth state, load the user
        let user = if let Some(ref state) = auth_state {
            match backend.get_user(&state.user_id).await {
                Ok(Some(user)) => Some(user),
                Ok(None) => {
                    // User was deleted - clear the session
                    let _ = session.delete().await;
                    None
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to load user from session");
                    None
                }
            }
        } else {
            None
        };

        // If user load failed, clear auth state
        let auth_state = if user.is_some() { auth_state } else { None };

        Self {
            user,
            state: auth_state,
            session,
            backend,
        }
    }

    /// Log in a user, creating session state
    ///
    /// Creates a new session with the user's current verification status.
    pub async fn login(&mut self, user: &B::User) -> Result<(), AuthError> {
        let state = AuthSessionState::new(user.id(), user.email_verified());

        self.session.insert(SESSION_KEY, &state).await?;
        self.user = Some(user.clone());
        self.state = Some(state);

        // Call backend hook
        self.backend.on_login(user).await?;

        Ok(())
    }

    /// Log in with device/IP metadata
    pub async fn login_with_metadata(
        &mut self,
        user: &B::User,
        device: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(), AuthError> {
        let state =
            AuthSessionState::new(user.id(), user.email_verified()).with_metadata(device, ip_address);

        self.session.insert(SESSION_KEY, &state).await?;
        self.user = Some(user.clone());
        self.state = Some(state);

        self.backend.on_login(user).await?;

        Ok(())
    }

    /// Log out, destroying the session
    pub async fn logout(&mut self) -> Result<(), AuthError> {
        if let Some(state) = &self.state {
            self.backend.on_logout(&state.user_id).await?;
        }

        self.session.delete().await?;
        self.user = None;
        self.state = None;

        Ok(())
    }

    /// Mark TOTP as verified for this session
    ///
    /// Call this after successful 2FA verification.
    pub async fn mark_totp_verified(&mut self) -> Result<(), AuthError> {
        if let Some(state) = &mut self.state {
            state.mark_totp_verified();
            self.session.insert(SESSION_KEY, state).await?;
        }
        Ok(())
    }

    /// Mark as recently re-authenticated
    ///
    /// Call this after the user confirms their password.
    pub async fn mark_reauthenticated(&mut self) -> Result<(), AuthError> {
        if let Some(state) = &mut self.state {
            state.mark_reauthenticated();
            self.session.insert(SESSION_KEY, state).await?;
        }
        Ok(())
    }

    /// Update the cached ban status
    pub async fn update_ban_status(
        &mut self,
        status: &crate::traits::BanStatus,
    ) -> Result<(), AuthError> {
        if let Some(state) = &mut self.state {
            state.update_ban_status(status);
            self.session.insert(SESSION_KEY, state).await?;
        }
        Ok(())
    }

    /// Refresh verification status from current user state
    pub async fn refresh_verification(&mut self) -> Result<(), AuthError> {
        if let (Some(user), Some(state)) = (&self.user, &mut self.state) {
            state.refresh_verification(user.email_verified());
            self.session.insert(SESSION_KEY, state).await?;
        }
        Ok(())
    }

    /// Touch the session (update last_seen)
    pub async fn touch(&mut self) -> Result<(), AuthError> {
        if let Some(state) = &mut self.state {
            state.touch();
            self.session.insert(SESSION_KEY, state).await?;
        }
        Ok(())
    }

    /// Get the auth backend
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Check if a user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    /// Get the user, returning an error if not authenticated
    pub fn user_required(&self) -> Result<&B::User, AuthError> {
        self.user
            .as_ref()
            .ok_or_else(|| AuthError::new(AuthErrorCode::Unauthenticated))
    }

    /// Get the session state, returning an error if not authenticated
    pub fn state_required(
        &self,
    ) -> Result<&AuthSessionState<<B::User as AuthUser>::Id>, AuthError> {
        self.state
            .as_ref()
            .ok_or_else(|| AuthError::new(AuthErrorCode::Unauthenticated))
    }
}

impl<S, B> FromRequestParts<S> for AuthSession<B>
where
    B: AuthBackend + FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the tower-sessions Session
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                AuthError::new(AuthErrorCode::SessionError)
                    .with_message("Failed to extract session")
            })?;

        // Get the backend from app state
        let backend = B::from_ref(state);

        // Try to load auth state from session
        let auth_state: Option<AuthSessionState<<B::User as AuthUser>::Id>> =
            session.get(SESSION_KEY).await?;

        // If we have auth state, load the user
        let user = if let Some(ref state) = auth_state {
            match backend.get_user(&state.user_id).await {
                Ok(Some(user)) => {
                    // Verify session auth hash hasn't changed (password change invalidates session)
                    // This is optional - implement if needed
                    Some(user)
                }
                Ok(None) => {
                    // User was deleted - clear the session
                    let _ = session.delete().await;
                    None
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to load user from session");
                    None
                }
            }
        } else {
            None
        };

        // If user load failed, clear auth state
        let auth_state = if user.is_some() { auth_state } else { None };

        Ok(Self {
            user,
            state: auth_state,
            session,
            backend,
        })
    }
}
