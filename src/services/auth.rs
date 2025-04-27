use async_trait::async_trait;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_login::{AuthUser, AuthnBackend, UserId};
use deadpool_diesel::postgres::Pool;
use password_auth::verify_password;
use serde::Deserialize;
use serde_json::json;
use tokio::task;

use crate::{db::models::user::User, modules::auth_v1::validator::V1LoginPayload};

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

#[derive(Clone)]
pub struct AuthBackend {
    pub pool: Pool,
}

impl AuthBackend {
    pub fn new(pool: &Pool) -> Self {
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
        let (status, error_message) = match &self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::PasswordVerificationError => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::SessionExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::DatabaseError(err) => {
                eprintln!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
            AuthError::InternalError(err) => {
                eprintln!("Internal error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "code": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

impl AuthUser for User {
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
    DatabaseError(#[from] crate::db::errors::DBError),

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
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;


    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::Password(password_creds) => {
                // Find user by email
                let user_result = User::find_by_email(&self.pool, password_creds.email).await;
                
                let user = match user_result {
                    Ok(Some(user)) => user,
                    Ok(None) => {
                        // Don't reveal whether the user exists or not for security reasons
                        return Ok(None);
                    },
                    Err(err) => {
                        return Err(AuthError::DatabaseError(err));
                    }
                };

                // Verify password
                let password = user.password.clone();
                let password_valid = match task::spawn_blocking(move || {
                    Self::check_password(password_creds.password, &password)
                })
                .await {
                    Ok(result) => result?,
                    Err(join_err) => {
                        return Err(AuthError::InternalError(format!("Task join error: {}", join_err)));
                    }
                };

                if password_valid {
                    Ok(Some(user))
                } else {
                    // Don't reveal that the password was incorrect (as opposed to user not existing)
                    Ok(None)
                }
            } // Add other credential types here if needed
        }
    }

    /// Retrieves a user by ID from the database.
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        User::find_by_id(&self.pool, *user_id).await
            .map_err(|err| {
                // Log the error for debugging purposes
                eprintln!("Error retrieving user {}: {:?}", user_id, err);
                AuthError::DatabaseError(err)
            })
    }
}
