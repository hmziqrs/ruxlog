//! Generic billing provider trait.
//!
//! Every payment integration (Stripe, Polar.sh, LemonSqueezy, Paddle, Crypto)
//! implements this trait. The admin panel and checkout flows work through this
//! abstraction, so adding a new provider requires only implementing this trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Result of creating a checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    /// Provider-specific session ID
    pub session_id: String,
    /// URL the customer should be redirected to
    pub checkout_url: String,
}

/// Result of a subscription lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub provider_subscription_id: String,
    pub status: String,
    pub current_period_end: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub cancel_at_period_end: bool,
}

/// Result of a payment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub provider_payment_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
}

/// Incoming webhook event from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Provider that sent this event
    pub provider: String,
    /// Raw payload bytes
    pub payload: Vec<u8>,
    /// Signature header value
    pub signature: String,
}

/// Parsed webhook after verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedWebhook {
    /// Event type (e.g., "subscription.created", "payment.succeeded")
    pub event_type: String,
    /// Provider customer ID
    pub customer_id: String,
    /// Provider subscription ID (if subscription event)
    pub subscription_id: Option<String>,
    /// Provider payment ID (if payment event)
    pub payment_id: Option<String>,
    /// Raw event data as JSON
    pub data: serde_json::Value,
}

/// Common billing operations every provider must support.
#[async_trait]
pub trait BillingProvider: Send + Sync {
    /// Name of this provider (e.g., "stripe", "polar").
    fn provider_name(&self) -> &'static str;

    /// Create a checkout session for a plan.
    async fn create_checkout(
        &self,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError>;

    /// Cancel a subscription at the provider.
    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError>;

    /// Get subscription info from the provider.
    async fn get_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError>;

    /// Verify and parse an incoming webhook.
    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError>;

    /// Create a billing portal session for the customer to manage their subscription.
    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError>;
}

/// Errors from billing operations.
#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider API error: {0}")]
    ProviderApi(String),

    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("{0}")]
    Other(String),
}
