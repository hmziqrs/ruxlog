use deadpool_diesel::{InteractError, PoolError};

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error("Database connection error {0:?}")]
    ConnectionError(#[from] PoolError),

    #[error("Database connection error")]
    UnknownError(#[from] InteractError),

    #[error("Password hashing error")]
    PasswordHashError,

    #[error("Database query error {0:?}")]
    QueryError(#[from] diesel::result::Error),
}
