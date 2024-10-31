use axum::{async_trait, http::StatusCode, response::IntoResponse, Json};
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
            .map_err(|_| AuthError::InvalidPassword)
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
        let status = match self {
            AuthError::InvalidPassword => StatusCode::UNAUTHORIZED,
            AuthError::InternalDBError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::UserNotFound => StatusCode::NOT_FOUND,
            AuthError::UnAuthorized => StatusCode::UNAUTHORIZED,
            // AuthError::DBXError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
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
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Internal DB error")]
    InternalDBError,

    #[error("User ID not found")]
    UserNotFound,

    #[error("Unauthorized access denied")]
    UnAuthorized,
    // #[error("Database error: {0:?}")]
    // DBXError(#[from] crate::db::errors::DBError),
}

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
                let user = User::find_by_email(&self.pool, password_creds.email).await;

                if let Err(err) = user {
                    eprintln!("Error: {:?}", err);
                    return Err(AuthError::InternalDBError);
                }

                if let Ok(Some(user)) = user {
                    let password = user.password.clone();
                    let password_valid = task::spawn_blocking(move || {
                        Self::check_password(password_creds.password, &password)
                    })
                    .await
                    .map_err(|_| AuthError::InvalidPassword)??;

                    if password_valid {
                        Ok(Some(user))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            } // Add other credential types here if needed
        }
    }

    /// Retrieves a user by ID from the database.
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        match User::find_by_id(&self.pool, *user_id).await {
            Ok(user) => Ok(user),
            Err(err) => {
                eprintln!("get_user Error: {:?}", err);
                Err(AuthError::InternalDBError)
            }
        }
    }
}
