//! OAuth provider abstractions

mod csrf;
mod google;
mod provider;

pub use csrf::CsrfStorage;
pub use google::{GoogleProvider, GoogleUserInfo};
pub use provider::{OAuthProvider, OAuthProviderConfig, OAuthUserHandler, OAuthUserInfo};
