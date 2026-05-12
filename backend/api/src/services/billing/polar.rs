//! Polar.sh billing provider integration.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Polar.sh billing provider.
pub struct PolarProvider {
    pub access_token: String,
    pub webhook_secret: String,
}

impl PolarProvider {
    pub fn new(access_token: String, webhook_secret: String) -> Self {
        Self {
            access_token,
            webhook_secret,
        }
    }
}

#[async_trait]
impl BillingProvider for PolarProvider {
    fn provider_name(&self) -> &'static str {
        "polar"
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
            "product_id": plan_slug,
            "customer_email": customer_email,
            "metadata": { "user_id": user_id },
            "success_url": success_url,
            "cancel_url": cancel_url,
        });

        let resp = client
            .post("https://api.polar.sh/v1/checkouts/")
            .header("Authorization", format!("Bearer {}", self.access_token))
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
            session_id: data["id"].as_str().unwrap_or_default().to_string(),
            checkout_url: data["url"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.polar.sh/v1/subscriptions/{}/cancel",
            provider_subscription_id
        );

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
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
            "https://api.polar.sh/v1/subscriptions/{}",
            provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
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

        Ok(SubscriptionInfo {
            provider_subscription_id: data["id"].as_str().unwrap_or_default().to_string(),
            status: data["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: data["current_period_end"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            cancel_at_period_end: data["cancel_at_period_end"].as_bool().unwrap_or(false),
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

        Ok(ParsedWebhook {
            event_type: data["type"].as_str().unwrap_or_default().to_string(),
            customer_id: data["data"]["customer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: data["data"]["subscription_id"]
                .as_str()
                .map(String::from),
            payment_id: data["data"]["order_id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        _provider_customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        // Polar.sh manages subscriptions through their own customer portal
        Err(BillingError::InvalidRequest(
            "Polar.sh uses its own customer portal".to_string(),
        ))
    }
}
