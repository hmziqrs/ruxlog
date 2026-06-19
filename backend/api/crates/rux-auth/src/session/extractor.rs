//! AuthSession extractor for Axum handlers

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use tower_sessions::Session;

use super::state::AuthSessionState;
use crate::error::{AuthError, AuthErrorCode};
use crate::traits::{AuthBackend, AuthUser};

/// Server-side session revocation check.
///
/// V-HIGH-2: `user_sessions.revoked_at` alone does NOT invalidate the live
/// tower-sessions record in the session store, so a revoked cookie keeps
/// authenticating until its inactivity expiry. Implementors that can reach a
/// revocation source (Redis set of revoked tower-session ids, or a DB lookup on
/// `revoked_at`) override [`SessionRevocation::is_session_revoked`]; the
/// extractor calls it on every authenticated request and, if it returns `true`,
/// deletes the session and treats the caller as unauthenticated.
///
/// The default implementation returns `false` so backends that opt out of
/// server-side revocation keep compiling unchanged.
///
/// **Fail-open policy:** on a backend error (e.g. a Redis blip) the extractor
/// logs a loud `warn!` and treats the session as *not* revoked. This mirrors the
/// rate limiter's fail-open behavior and avoids a mass lockout during a
/// transient revocation-store outage — at the cost of a revoked session briefly
/// staying live until the store recovers. The DB `revoked_at` stamp remains as
/// an audit record and a secondary signal regardless.
#[async_trait::async_trait]
pub trait SessionRevocation {
    /// Return `true` if the given tower-session id has been administratively
    /// revoked and must stop authenticating immediately.
    async fn is_session_revoked(&self, _tower_session_id: &str) -> Result<bool, AuthError> {
        Ok(false)
    }
}

/// Constant-time byte equality. Used when comparing the stored
/// `session_auth_hash` snapshot against the user's current hash so a timing
/// side-channel cannot leak bytes of the (password) hash. A length mismatch
/// returns `false` immediately — the length is fixed/public per scheme.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

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

impl<B: AuthBackend + SessionRevocation> AuthSession<B> {
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
                Ok(Some(user)) => {
                    // Invalidate if the credential changed since login (password
                    // reset/change) or the user identity was swapped underneath us.
                    if !ct_eq(&state.session_auth_hash, user.session_auth_hash()) {
                        tracing::warn!(
                            "Session auth hash mismatch — invalidating stale session"
                        );
                        let _ = session.delete().await;
                        None
                    } else if is_revoked(&backend, &session).await {
                        // V-HIGH-2: server-side revocation. The session was
                        // terminated out-of-band (e.g. via sessions_terminate);
                        // kill the live record so the cookie stops authenticating
                        // immediately.
                        tracing::info!(
                            user_id = ?state.user_id,
                            "Session revoked server-side — invalidating"
                        );
                        let _ = session.delete().await;
                        None
                    } else {
                        Some(user)
                    }
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

        Self {
            user,
            state: auth_state,
            session,
            backend,
        }
    }
}

/// Methods available for any [`AuthBackend`] (no revocation awareness required).
/// Revocation invalidation lives only in `new` (the `AuthBackend +
/// SessionRevocation` block above, which calls `is_session_revoked`); keeping
/// `backend()` / `update_ban_status()` / etc. here lets generic middleware
/// (the ban/role/TOTP checks in `check_requirements`, which is bounded only by
/// `AuthBackend`) compile.
impl<B: AuthBackend> AuthSession<B> {
    /// Borrow the underlying tower-sessions [`Session`].
    ///
    /// Exposed so callers can reach session-level primitives such as the session
    /// id — e.g. to bind an OAuth `state` token to the caller's session, closing
    /// the login-CSRF gap where a state issued to one session is replayed in
    /// another.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Log in a user, creating session state
    ///
    /// Creates a new session with the user's current verification status.
    pub async fn login(&mut self, user: &B::User) -> Result<(), AuthError> {
        let mut state = AuthSessionState::new(user.id(), user.email_verified());
        state.session_auth_hash = user.session_auth_hash().to_vec();

        // Rotate the session id on privilege change (anonymous → authenticated)
        // to defeat session fixation: an attacker who planted a known session
        // cookie cannot ride along after the victim authenticates. `cycle_id`
        // preserves the in-memory record's data while deleting the old id from
        // the store and arming a fresh id (persisted when the session saves at
        // the end of the response). The auth state is then written under the new
        // id, and the frontend re-fetches its CSRF token (bound to the new id).
        self.session.cycle_id().await?;
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
        let mut state = AuthSessionState::new(user.id(), user.email_verified())
            .with_metadata(device, ip_address);
        state.session_auth_hash = user.session_auth_hash().to_vec();

        // Rotate the session id on login — see `login()` for rationale.
        self.session.cycle_id().await?;
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

/// V-HIGH-2: consult the backend's revocation source for this tower-session id.
///
/// Returns `true` only when the backend *definitively* reports the session as
/// revoked. On a backend error the check **fails open** (returns `false`) and
/// logs a `warn!` — see [`SessionRevocation`] for the rationale (avoid mass
/// lockout on a revocation-store blip). The caller is still responsible for
/// deleting the session record when this returns `true`.
async fn is_revoked<B>(backend: &B, session: &Session) -> bool
where
    B: AuthBackend + SessionRevocation,
{
    // `session.id()` is `None` until the record has been saved at least once;
    // an unsaved session has nothing to revoke.
    let Some(id) = session.id() else {
        return false;
    };
    let tower_sid = id.to_string();
    match backend.is_session_revoked(&tower_sid).await {
        Ok(true) => true,
        Ok(false) => false,
        Err(e) => {
            // Fail-open: a Redis/DB blip must not lock out every user. The
            // `revoked_at` audit row and the session's own 14-day inactivity
            // expiry still bound the window.
            tracing::warn!(
                error = ?e,
                tower_session_id = %tower_sid,
                "Revocation check failed (fail-open): session remains valid until store recovers"
            );
            false
        }
    }
}

impl<S, B> FromRequestParts<S> for AuthSession<B>
where
    B: AuthBackend + SessionRevocation + FromRef<S>,
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
                    // Invalidate if the credential changed since login (password
                    // reset/change). See `AuthUser::session_auth_hash`.
                    if !ct_eq(&state.session_auth_hash, user.session_auth_hash()) {
                        tracing::warn!(
                            "Session auth hash mismatch — invalidating stale session"
                        );
                        let _ = session.delete().await;
                        None
                    } else if is_revoked(&backend, &session).await {
                        // V-HIGH-2: server-side revocation — see `new()`.
                        tracing::info!(
                            user_id = ?state.user_id,
                            "Session revoked server-side — invalidating"
                        );
                        let _ = session.delete().await;
                        None
                    } else {
                        Some(user)
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BanStatus;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    #[derive(Clone, Debug)]
    struct MockUser {
        id: i32,
        hash: Vec<u8>,
    }

    impl AuthUser for MockUser {
        type Id = i32;
        fn id(&self) -> Self::Id {
            self.id
        }
        fn session_auth_hash(&self) -> &[u8] {
            &self.hash
        }
        fn email_verified(&self) -> bool {
            true
        }
        fn totp_enabled(&self) -> bool {
            false
        }
        fn role_level(&self) -> i32 {
            0
        }
    }

    #[derive(Clone)]
    struct MockBackend;

    #[async_trait]
    impl AuthBackend for MockBackend {
        type User = MockUser;
        async fn get_user(&self, id: &i32) -> Result<Option<MockUser>, AuthError> {
            Ok(Some(MockUser {
                id: *id,
                hash: vec![1, 2, 3],
            }))
        }
        async fn check_ban(&self, _id: &i32) -> Result<BanStatus, AuthError> {
            Ok(BanStatus::NotBanned)
        }
        async fn verify_password(&self, _id: &i32, _password: &str) -> Result<bool, AuthError> {
            Ok(true)
        }
    }

    // Default (no-op) revocation check — `MockBackend` never revokes.
    impl SessionRevocation for MockBackend {}

    /// Backend whose revocation set is an in-memory `Mutex<HashSet>`. Mirrors the
    /// production Redis-backed check without requiring a live Redis.
    #[derive(Clone, Default)]
    struct RevocableBackend {
        revoked: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
    }

    #[async_trait]
    impl AuthBackend for RevocableBackend {
        type User = MockUser;
        async fn get_user(&self, id: &i32) -> Result<Option<MockUser>, AuthError> {
            Ok(Some(MockUser {
                id: *id,
                hash: vec![1, 2, 3],
            }))
        }
        async fn check_ban(&self, _id: &i32) -> Result<BanStatus, AuthError> {
            Ok(BanStatus::NotBanned)
        }
        async fn verify_password(&self, _id: &i32, _password: &str) -> Result<bool, AuthError> {
            Ok(true)
        }
    }

    #[async_trait]
    impl SessionRevocation for RevocableBackend {
        async fn is_session_revoked(&self, tower_session_id: &str) -> Result<bool, AuthError> {
            Ok(self
                .revoked
                .lock()
                .map(|g| g.contains(tower_session_id))
                .unwrap_or(false))
        }
    }

    /// Materialize an anonymous session with a known id and return it, ready to
    /// be handed to `AuthSession::new`. The `SessionManagerLayer` normally runs
    /// this lifecycle in production; here we drive it directly.
    async fn anon_session() -> Session {
        let store = Arc::new(MemoryStore::default());
        let session = Session::new(None, store, None);
        session.insert("anon", true).await.unwrap();
        session.save().await.unwrap();
        session
    }

    /// Logging in must rotate the session id so an attacker who planted a known
    /// session cookie cannot ride along after the victim authenticates.
    #[tokio::test]
    async fn login_rotates_the_session_id() {
        let session = anon_session().await;
        let id_before = session.id().expect("session id present after save");

        let mut auth: AuthSession<MockBackend> = AuthSession::new(MockBackend, session).await;
        auth.login(&MockUser {
            id: 42,
            hash: vec![9, 9, 9],
        })
        .await
        .unwrap();

        // `login` cycles the id but does not persist it (the layer saves at
        // response time). Save here to materialize the rotated id.
        auth.session().save().await.unwrap();
        let id_after = auth
            .session()
            .id()
            .expect("rotated session id present after save");
        assert_ne!(
            id_before, id_after,
            "login must rotate the session id (session-fixation defense)"
        );
    }

    /// Same guarantee on the metadata-bearing login path.
    #[tokio::test]
    async fn login_with_metadata_rotates_the_session_id() {
        let session = anon_session().await;
        let id_before = session.id().unwrap();

        let mut auth: AuthSession<MockBackend> = AuthSession::new(MockBackend, session).await;
        auth.login_with_metadata(
            &MockUser {
                id: 7,
                hash: vec![4, 5, 6],
            },
            Some("device".to_string()),
            Some("127.0.0.1".to_string()),
        )
        .await
        .unwrap();

        auth.session().save().await.unwrap();
        let id_after = auth.session().id().unwrap();
        assert_ne!(
            id_before, id_after,
            "login_with_metadata must rotate the session id"
        );
    }

    /// V-HIGH-2: after a session is marked revoked in the backend's revocation
    /// store, a subsequent request carrying the SAME cookie must come back
    /// unauthenticated. This is the regression test for "terminate session does
    /// not actually invalidate the live tower-sessions record".
    #[tokio::test]
    async fn revoked_session_no_longer_authenticates() {
        let backend = RevocableBackend::default();
        let session = anon_session().await;

        // Authenticate the session.
        let mut auth: AuthSession<RevocableBackend> =
            AuthSession::new(backend.clone(), session).await;
        auth.login(&MockUser {
            id: 5,
            hash: vec![1, 2, 3],
        })
        .await
        .unwrap();
        auth.session().save().await.unwrap();
        let tower_sid = auth
            .session()
            .id()
            .expect("session id present after save")
            .to_string();
        // The session was live and authenticated.
        assert!(auth.user.is_some(), "session must authenticate before revoke");

        // Drop auth but keep the SAME session record (the cookie).
        let session_record = auth.session().clone();
        drop(auth);

        // Out-of-band revoke (this is what sessions_terminate + Redis sadd do).
        {
            let mut g = backend.revoked.lock().unwrap();
            g.insert(tower_sid.clone());
        }

        // Next request: re-extract AuthSession from the same session record.
        let next: AuthSession<RevocableBackend> =
            AuthSession::new(backend, session_record).await;

        // The user MUST now be None — the cookie stopped authenticating.
        assert!(
            next.user.is_none(),
            "a revoked session must not authenticate on the next request"
        );
    }

    /// V-HIGH-2 fail-open: if the revocation source errors, the session stays
    /// valid rather than mass-locking users on a store blip.
    #[tokio::test]
    async fn revocation_check_failure_is_fail_open() {
        struct ErroringBackend;
        #[async_trait]
        impl AuthBackend for ErroringBackend {
            type User = MockUser;
            async fn get_user(&self, id: &i32) -> Result<Option<MockUser>, AuthError> {
                Ok(Some(MockUser {
                    id: *id,
                    hash: vec![1, 2, 3],
                }))
            }
            async fn check_ban(&self, _id: &i32) -> Result<BanStatus, AuthError> {
                Ok(BanStatus::NotBanned)
            }
            async fn verify_password(&self, _id: &i32, _password: &str) -> Result<bool, AuthError> {
                Ok(true)
            }
        }
        #[async_trait]
        impl SessionRevocation for ErroringBackend {
            async fn is_session_revoked(
                &self,
                _tower_session_id: &str,
            ) -> Result<bool, AuthError> {
                Err(AuthError::new(AuthErrorCode::BackendError)
                    .with_message("revocation store unavailable"))
            }
        }

        let session = anon_session().await;
        let mut auth: AuthSession<ErroringBackend> =
            AuthSession::new(ErroringBackend, session).await;
        auth.login(&MockUser {
            id: 9,
            hash: vec![1, 2, 3],
        })
        .await
        .unwrap();
        auth.session().save().await.unwrap();
        let live = auth.session().clone();
        drop(auth);

        let next: AuthSession<ErroringBackend> = AuthSession::new(ErroringBackend, live).await;
        assert!(
            next.user.is_some(),
            "revocation-store error must fail OPEN, not lock the user out"
        );
    }
}
