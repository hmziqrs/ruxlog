use deadpool_diesel::{InteractError, PoolError};

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error("Database connection error")]
    ConnectionError(#[from] PoolError),

    #[error("Database connection error")]
    UnknownError(#[from] InteractError),

    #[error("Database query error {0:?}")]
    QueryError(#[from] diesel::result::Error),
}
