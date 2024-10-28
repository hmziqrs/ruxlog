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

    #[error("Unauthorized! access denied")]
    UnAuthorized,
}

impl AuthBackend {
    /// Create a new backend instance.
    pub fn new(pool: &Pool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Verify a password against a hash.
    fn check_password(password: String, hash: &str) -> Result<bool, AuthError> {
        verify_password(password, hash)
            .map(|_| true)
            .map_err(|_| AuthError::InvalidPassword)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum Credentials {
    Password(V1LoginPayload),
    // OAuth(OAuthCreds),
}

// #[async_trait]
// impl AuthzBackend for AuthBackend {
//     type Permission = UserRole;

//     async fn get_user_permissions(
//         &self,
//         user: &Self::User,
//     ) -> Result<HashSet<Self::Permission>, Self::Error> {
//         let permissions = vec![UserRole::from_str(&user.role).unwrap()];
//         Ok(permissions.into_iter().collect())
//     }

//     async fn has_perm(
//         &self,
//         user: &Self::User,
//         perm: Self::Permission,
//     ) -> Result<bool, Self::Error> {
//         match UserRole::from_str(&user.role) {
//             Ok(user_perm) => {
//                 if user_perm.to_i32() >= perm.to_i32() {
//                     Ok(true)
//                 } else {
//                     Ok(false)
//                 }
//             }
//             Err(_) => Err(AuthError::UnAuthorized),
//         }
//     }
// }

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
                let user = User::find_by_email(&self.pool, password_creds.email)
                    .await
                    .map_err(|_| AuthError::InternalDBError)?;

                if let Some(user) = user {
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
        User::find_by_id(&self.pool, *user_id)
            .await
            .map_err(|_| AuthError::UserNotFound)
    }
}
