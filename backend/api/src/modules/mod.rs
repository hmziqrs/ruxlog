// Always enabled (core)
pub mod auth_v1;
pub mod category_v1;
pub mod csrf_v1;
pub mod feed_v1;
pub mod media_v1;
pub mod post_v1;
pub mod super_admin_v1;
pub mod tag_v1;

// Feature-gated
#[cfg(feature = "analytics")]
pub mod analytics_v1;

#[cfg(feature = "auth-oauth")]
pub mod google_auth_v1;

#[cfg(feature = "user-management")]
pub mod email_verification_v1;

#[cfg(feature = "user-management")]
pub mod forgot_password_v1;

#[cfg(feature = "user-management")]
pub mod user_v1;

#[cfg(feature = "comments")]
pub mod post_comment_v1;

#[cfg(feature = "newsletter")]
pub mod newsletter_v1;

#[cfg(feature = "admin-acl")]
pub mod admin_acl_v1;

#[cfg(feature = "admin-routes")]
pub mod admin_route_v1;

#[cfg(feature = "seed-system")]
pub mod seed_v1;
