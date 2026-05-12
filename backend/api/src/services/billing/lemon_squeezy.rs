//! LemonSqueezy billing provider integration.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// LemonSqueezy billing provider.
pub struct LemonSqueezyProvider {
    pub api_key: String,
    pub webhook_secret: String,
    pub store_id: String,
}

impl LemonSqueezyProvider {
    pub fn new(api_key: String, webhook_secret: String, store_id: String) -> Self {
        Self {
            api_key,
            webhook_secret,
            store_id,
        }
    }
}

#[async_trait]
impl BillingProvider for LemonSqueezyProvider {
    fn provider_name(&self) -> &'static str {
        "lemon_squeezy"
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
            "data": {
                "type": "checkouts",
                "attributes": {
                    "checkout_data": {
                        "email": customer_email,
                        "custom": { "user_id": user_id.to_string() }
                    },
                    "product_options": {},
                    "urls": {
                        "success_url": success_url,
                        "cancel_url": cancel_url,
                    }
                },
                "relationships": {
                    "store": { "data": { "type": "stores", "id": self.store_id } },
                    "variant": { "data": { "type": "variants", "id": plan_slug } }
                }
            }
        });

        let resp = client
            .post("https://api.lemonsqueezy.com/v1/checkouts")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/vnd.api+json")
            .header("Content-Type", "application/vnd.api+json")
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

        let attrs = &data["data"]["attributes"];
        Ok(CheckoutSession {
            session_id: data["data"]["id"].as_str().unwrap_or_default().to_string(),
            checkout_url: attrs["url"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.lemonsqueezy.com/v1/subscriptions/{}",
            provider_subscription_id
        );
        let body = serde_json::json!({
            "data": {
                "type": "subscriptions",
                "id": provider_subscription_id,
                "attributes": { "cancelled": true }
            }
        });

        let resp = client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/vnd.api+json")
            .header("Content-Type", "application/vnd.api+json")
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
            "https://api.lemonsqueezy.com/v1/subscriptions/{}",
            provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/vnd.api+json")
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

        let attrs = &data["data"]["attributes"];
        Ok(SubscriptionInfo {
            provider_subscription_id: data["data"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            status: attrs["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: attrs["renews_at"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            cancel_at_period_end: attrs["cancelled"]
                .as_bool()
                .unwrap_or(false),
        })
    }

    async fn verify_webhook(
        &self,
        event: WebhookEvent,
    ) -> Result<ParsedWebhook, BillingError> {
        let payload_str = String::from_utf8(event.payload.clone())
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let obj = &data["data"]["attributes"];

        Ok(ParsedWebhook {
            event_type: data["meta"]["event_name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            customer_id: obj["customer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: data["data"]["id"].as_str().map(String::from),
            payment_id: obj["order_id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        _provider_customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        Err(BillingError::InvalidRequest(
            "LemonSqueezy uses its own customer portal".to_string(),
        ))
    }
}
