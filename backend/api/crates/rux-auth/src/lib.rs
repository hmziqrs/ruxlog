//! rux-auth: Composable authentication and authorization for Axum
//!
//! Provides:
//! - Rich session state (not just user_id)
//! - Composable requirement middleware
//! - Inverse route guards (unauthenticated/unverified only)
//! - OAuth provider abstraction
//! - Trait-based integration
//!
//! # Quick Start
//!
//! ```ignore
//! use rux_auth::{AuthSession, AuthBackend, AuthUser, auth_requirements, auth_guard};
//!
//! // Implement AuthUser on your user model
//! impl AuthUser for User {
//!     type Id = i32;
//!     fn id(&self) -> i32 { self.id }
//!     // ... other methods
//! }
//!
//! // Implement AuthBackend
//! impl AuthBackend for MyBackend {
//!     type User = User;
//!     // ... methods
//! }
//!
//! // Use in routes
//! Router::new()
//!     .route("/login", post(login))
//!     .layer(auth_guard::<MyBackend>(auth_requirements().unauthenticated()))
//! ```

pub mod error;
pub mod middleware;
pub mod oauth;
pub mod requirements;
pub mod session;
pub mod traits;

// Core exports
pub use error::{AuthError, AuthErrorCode};
pub use traits::{AuthBackend, AuthUser, BanStatus};

// Session exports
pub use session::{AuthSession, AuthSessionState};

// Requirements exports
pub use requirements::{auth_requirements, AuthRequirements};

// Middleware exports
pub use middleware::{auth_guard, auth_guard_fn, check_requirements, AuthGuard, AuthGuardLayer};

// OAuth exports
pub use oauth::{
    CsrfStorage, GoogleProvider, GoogleUserInfo, OAuthProvider, OAuthProviderConfig,
    OAuthUserHandler, OAuthUserInfo,
};
