// Always enabled
pub mod abuse_limiter;
pub mod auth;
pub mod mail;
pub mod redis;

// Feature-gated
#[cfg(feature = "image-optimization")]
pub mod image_optimizer;

#[cfg(feature = "admin-acl")]
pub mod acl_service;

#[cfg(feature = "admin-routes")]
pub mod route_blocker_config;

#[cfg(feature = "admin-routes")]
pub mod route_blocker_service;

#[cfg(feature = "seed-system")]
pub mod seed;

#[cfg(feature = "seed-system")]
pub mod seed_config;
