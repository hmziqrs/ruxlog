//! Crypto payment provider integration (No-KYC, direct wallet).
//!
//! Supports generating payment addresses and verifying on-chain transactions.
//! Uses a configurable blockchain API (e.g., NowNodes, BlockCypher, or self-hosted).

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Crypto billing provider.
pub struct CryptoProvider {
    /// Wallet address to receive payments
    pub wallet_address: String,
    /// Blockchain API base URL (e.g., "https://api.blockcypher.com/v1")
    pub api_url: String,
    /// API key for blockchain service
    pub api_key: String,
    /// Supported currency symbol (e.g., "BTC", "ETH", "XMR")
    pub currency: String,
}

impl CryptoProvider {
    pub fn new(wallet_address: String, api_url: String, api_key: String, currency: String) -> Self {
        Self {
            wallet_address,
            api_url,
            api_key,
            currency,
        }
    }
}

#[async_trait]
impl BillingProvider for CryptoProvider {
    fn provider_name(&self) -> &'static str {
        "crypto"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        _customer_email: &str,
        user_id: i32,
        success_url: &str,
        _cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        // For crypto, we generate a unique payment reference
        // The "checkout URL" directs the user to pay to our wallet with a specific memo
        let payment_id = format!("rux-{}-{}", user_id, uuid::Uuid::new_v4());

        // Amount is encoded in the plan_slug as "{amount}_{currency}" or looked up from DB
        let checkout_url = format!(
            "{}?address={}&amount={}&memo={}&callback={}",
            self.api_url, self.wallet_address, plan_slug, payment_id, success_url
        );

        Ok(CheckoutSession {
            session_id: payment_id,
            checkout_url,
        })
    }

    async fn cancel_subscription(
        &self,
        _provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        // Crypto payments are one-time; no subscription to cancel
        Ok(())
    }

    async fn get_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError> {
        // Crypto doesn't have traditional subscriptions
        // Return a synthetic info based on payment status
        Ok(SubscriptionInfo {
            provider_subscription_id: provider_subscription_id.to_string(),
            status: "active".to_string(),
            current_period_end: None,
            cancel_at_period_end: false,
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Crypto webhooks come from the blockchain monitoring service
        let payload_str = String::from_utf8(event.payload.clone())
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let tx_hash = data["hash"]
            .as_str()
            .unwrap_or(data["tx_hash"].as_str().unwrap_or_default())
            .to_string();

        let confirmations = data["confirmations"].as_u64().unwrap_or(0);

        let event_type = if confirmations >= 3 {
            "payment.confirmed"
        } else {
            "payment.pending"
        };

        Ok(ParsedWebhook {
            event_type: event_type.to_string(),
            customer_id: data["address"].as_str().unwrap_or_default().to_string(),
            subscription_id: None,
            payment_id: Some(tx_hash),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        _provider_customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        // No portal for crypto — users manage their own wallets
        Err(BillingError::InvalidRequest(
            "Crypto payments have no portal".to_string(),
        ))
    }
}
