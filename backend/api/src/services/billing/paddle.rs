//! Paddle billing provider integration.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Paddle billing provider.
pub struct PaddleProvider {
    pub client_token: String,
    pub webhook_secret: String,
}

impl PaddleProvider {
    pub fn new(client_token: String, webhook_secret: String) -> Self {
        Self {
            client_token,
            webhook_secret,
        }
    }

    pub fn from_env() -> Result<Self, BillingError> {
        let client_token = std::env::var("PADDLE_CLIENT_TOKEN")
            .map_err(|_| BillingError::Config("PADDLE_CLIENT_TOKEN not set".into()))?;
        let webhook_secret = std::env::var("PADDLE_WEBHOOK_SECRET")
            .map_err(|_| BillingError::Config("PADDLE_WEBHOOK_SECRET not set".into()))?;
        Ok(Self::new(client_token, webhook_secret))
    }
}

#[async_trait]
impl BillingProvider for PaddleProvider {
    fn provider_name(&self) -> &'static str {
        "paddle"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "items": [{ "price_id": plan_slug, "quantity": 1 }],
            "custom_data": { "user_id": user_id.to_string() },
            "customer_email": customer_email,
            "urls": {
                "success_url": success_url,
                "cancel_url": cancel_url,
            }
        });

        let resp = client
            .post("https://api.paddle.com/transactions")
            .header("Authorization", format!("Bearer {}", self.client_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(BillingError::ProviderApi(body));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        Ok(CheckoutSession {
            session_id: data["data"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            checkout_url: data["data"]["checkout"]["url"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.paddle.com/subscriptions/{}",
            provider_subscription_id
        );

        let body = if immediately {
            serde_json::json!({ "status": "canceled" })
        } else {
            serde_json::json!({ "scheduled_change": { "action": "cancel" } })
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.client_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(BillingError::ProviderApi(body));
        }

        Ok(())
    }

    async fn get_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.paddle.com/subscriptions/{}",
            provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.client_token))
            .send()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(BillingError::SubscriptionNotFound(
                provider_subscription_id.to_string(),
            ));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        let d = &data["data"];
        Ok(SubscriptionInfo {
            provider_subscription_id: d["id"].as_str().unwrap_or_default().to_string(),
            status: d["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: d["next_billed_at"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            cancel_at_period_end: d["scheduled_change"]["action"]
                .as_str()
                .map(|a| a == "cancel")
                .unwrap_or(false),
        })
    }

    async fn verify_webhook(
        &self,
        event: WebhookEvent,
    ) -> Result<ParsedWebhook, BillingError> {
        // Verify Paddle webhook signature using HMAC-SHA256
        if let Some(signature) = event.headers.get("paddle-signature") {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;

            type HmacSha256 = Hmac<Sha256>;

            let payload_str = String::from_utf8(event.payload.clone())
                .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

            let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
                .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
            mac.update(payload_str.as_bytes());
            let expected = hex::encode(mac.finalize().into_bytes());

            if !constant_time_eq_str(&expected, signature) {
                return Err(BillingError::WebhookVerification(
                    "Invalid Paddle webhook signature".into(),
                ));
            }
        }

        let payload_str = String::from_utf8(event.payload.clone())
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let obj = &data["data"];

        Ok(ParsedWebhook {
            event_type: data["event_type"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            customer_id: obj["customer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: obj["id"].as_str().map(String::from),
            payment_id: obj["transaction_id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        _provider_customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        Err(BillingError::InvalidRequest(
            "Paddle uses its own customer portal".to_string(),
        ))
    }
}

fn constant_time_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    a_bytes.iter().zip(b_bytes.iter()).fold(0, |acc, (x, y)| {
        acc | (x ^ y)
    }) == 0
}
