//! Billing and monetization service layer.
//!
//! All payment providers implement the `BillingProvider` trait.
//! The `BillingRouter` holds all initialized providers and routes
//! requests by geo or provider name. Feature-gated behind `billing`.

pub mod provider;
pub mod router;

pub mod airwallex;
pub mod crypto;
pub mod lemon_squeezy;
pub mod mercado_pago;
pub mod paddle;
pub mod paypal;
pub mod polar;
pub mod razorpay;
pub mod revolut;
pub mod stripe;

pub use provider::BillingProvider;
pub use router::BillingRouter;
