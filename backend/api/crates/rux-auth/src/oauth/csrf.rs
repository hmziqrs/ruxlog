//! CSRF token storage for OAuth flows

use async_trait::async_trait;

use crate::error::AuthError;

/// CSRF token storage trait
///
/// Implement this to store CSRF tokens for OAuth state validation.
/// Typically backed by Redis or another fast key-value store.
#[async_trait]
pub trait CsrfStorage: Clone + Send + Sync + 'static {
    /// Store a CSRF token with a TTL
    ///
    /// The token should expire after `ttl_seconds` to prevent replay attacks.
    async fn store(&self, token: &str, ttl_seconds: u64) -> Result<(), AuthError>;

    /// Verify and consume a CSRF token
    ///
    /// Returns `true` if the token was valid and has been consumed.
    /// Returns `false` if the token was invalid or already used.
    ///
    /// Important: This should be atomic - the token should be deleted
    /// immediately after verification to prevent reuse.
    async fn verify_and_consume(&self, token: &str) -> Result<bool, AuthError>;
}
