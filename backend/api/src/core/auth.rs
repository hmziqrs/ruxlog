use std::sync::Arc;

use password_auth::verify_password;
use tokio::task;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    core::{
        context::CoreContext,
        types::{AuthError, Session, UserCredentials},
    },
    db::sea_models::user,
};

#[derive(Clone)]
pub struct AuthService {
    core: Arc<CoreContext>,
}

impl AuthService {
    pub fn new(core: Arc<CoreContext>) -> Self {
        Self { core }
    }

    #[instrument(skip(self, creds), fields(username = %creds.username))]
    pub async fn login(&self, creds: UserCredentials) -> Result<Session, AuthError> {
        let user = user::Entity::find_by_email(&self.core.db, creds.username.clone())
            .await
            .map_err(|err| {
                error!(error = ?err, "Database error during user lookup (core auth)");
                AuthError::Internal(err.to_string())
            })?;

        let user = match user {
            Some(user) => user,
            None => {
                warn!("User not found (core auth)");
                return Err(AuthError::UserNotFound);
            }
        };

        let password_hash = match &user.password {
            Some(hash) => hash.clone(),
            None => {
                warn!("User has no password hash (likely OAuth-only)");
                return Err(AuthError::InvalidCredentials);
            }
        };

        let password = creds.password;
        let verify_result = task::spawn_blocking(move || verify_password(password, &password_hash))
            .await
            .map_err(|join_err| {
                error!(
                    error = %join_err,
                    "Password verification task failed (core auth)"
                );
                AuthError::PasswordVerificationError
            })?;

        // password_auth::verify_password returns Result<(), Error>
        if let Err(err) = verify_result {
            warn!(error = ?err, "Invalid password (core auth)");
            return Err(AuthError::InvalidCredentials);
        }

        info!(user_id = user.id, "Authentication successful (core auth)");

        // For the TUI we only need an in-process session; we generate
        // a simple opaque ID. Redis/integration can be added later.
        let session = Session {
            session_id: Uuid::new_v4().to_string(),
            user_id: user.id,
        };

        Ok(session)
    }

    pub async fn logout(&self, _session: Session) -> Result<(), AuthError> {
        // For now there is no persistent session store to clean up.
        // This hook exists for future Redis-backed session management.
        Ok(())
    }
}

