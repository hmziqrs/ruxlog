//! Billing and monetization service layer.
//!
//! Each payment provider implements the `BillingProvider` trait.
//! Feature-gated behind the `billing` Cargo feature.

pub mod provider;

#[cfg(feature = "billing-stripe")]
pub mod stripe;

#[cfg(feature = "billing-polar")]
pub mod polar;

#[cfg(feature = "billing-lemonsqueezy")]
pub mod lemon_squeezy;

#[cfg(feature = "billing-paddle")]
pub mod paddle;

#[cfg(feature = "billing-crypto")]
pub mod crypto;

pub use provider::BillingProvider;
