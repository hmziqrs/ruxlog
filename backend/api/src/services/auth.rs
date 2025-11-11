use async_trait::async_trait;
use axum::response::IntoResponse;
use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::verify_password;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::user, error::ErrorResponse, modules::auth_v1::validator::V1LoginPayload,
    utils::telemetry,
};

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

#[derive(Clone)]
pub struct AuthBackend {
    pub pool: DatabaseConnection,
}

impl AuthBackend {
    pub fn new(pool: &DatabaseConnection) -> Self {
        Self { pool: pool.clone() }
    }
    fn check_password(password: String, hash: &str) -> Result<bool, AuthError> {
        verify_password(password, hash)
            .map(|_| true)
            .map_err(|_| AuthError::PasswordVerificationError)
    }
}

impl std::fmt::Debug for AuthBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthBackend")
            .field("pool", &"Pool{...}")
            .finish()
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        // Convert AuthError to our standard ErrorResponse using the conversion
        let error_response: ErrorResponse = self.into();
        error_response.into_response()
    }
}

impl AuthUser for user::Model {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Password verification failed")]
    PasswordVerificationError,

    #[error("User not found")]
    UserNotFound,

    #[error("Unauthorized access denied")]
    Unauthorized,

    #[error("Session expired")]
    SessionExpired,

    #[error("Database error: {0}")]
    DatabaseError(crate::error::ErrorResponse),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

// Make the AuthError type safe to share between threads
unsafe impl Sync for AuthError {}
unsafe impl Send for AuthError {}

#[derive(Debug, Clone, Deserialize)]
pub enum Credentials {
    Password(V1LoginPayload),
}

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = user::Model;
    type Credentials = Credentials;
    type Error = AuthError;

    #[instrument(skip(self, creds), fields(email = %"<redacted>", result))]
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let metrics = telemetry::auth_metrics();
        metrics.login_attempts.add(1, &[]);

        match creds {
            Credentials::Password(password_creds) => {
                let email = password_creds.email.clone();
                tracing::Span::current().record("email", &email);

                info!("Attempting authentication");

                let user_result =
                    user::Entity::find_by_email(&self.pool, password_creds.email).await;

                let user = match user_result {
                    Ok(Some(user)) => user,
                    Ok(None) => {
                        warn!("User not found");
                        tracing::Span::current().record("result", "user_not_found");
                        metrics.login_failure.add(
                            1,
                            &[opentelemetry::KeyValue::new("reason", "user_not_found")],
                        );
                        return Ok(None);
                    }
                    Err(err) => {
                        error!(error = ?err, "Database error during user lookup");
                        metrics
                            .login_failure
                            .add(1, &[opentelemetry::KeyValue::new("reason", "db_error")]);
                        return Err(AuthError::DatabaseError(err));
                    }
                };

                // Verify password
                let password = user.password.clone();
                let verify_start = Instant::now();
                let password_valid = match task::spawn_blocking(move || {
                    Self::check_password(password_creds.password, &password)
                })
                .await
                {
                    Ok(result) => result?,
                    Err(join_err) => {
                        error!(error = %join_err, "Password verification task failed");
                        metrics
                            .login_failure
                            .add(1, &[opentelemetry::KeyValue::new("reason", "task_error")]);
                        return Err(AuthError::InternalError(format!(
                            "Task join error: {}",
                            join_err
                        )));
                    }
                };

                let verify_duration = verify_start.elapsed().as_millis() as f64;
                metrics
                    .password_verification_duration
                    .record(verify_duration, &[]);

                if password_valid {
                    info!(user_id = user.id, "Authentication successful");
                    tracing::Span::current().record("result", "success");
                    metrics.login_success.add(1, &[]);
                    metrics.session_created.add(1, &[]);
                    Ok(Some(user))
                } else {
                    warn!("Invalid password");
                    tracing::Span::current().record("result", "invalid_password");
                    metrics.login_failure.add(
                        1,
                        &[opentelemetry::KeyValue::new("reason", "invalid_password")],
                    );
                    Ok(None)
                }
            } // Add other credential types here if needed
        }
    }

    /// Retrieves a user by ID from the database.
    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        user::Entity::get_by_id(&self.pool, *user_id)
            .await
            .map_err(|err| {
                error!(error = ?err, "Error retrieving user");
                AuthError::DatabaseError(err)
            })
    }
}
