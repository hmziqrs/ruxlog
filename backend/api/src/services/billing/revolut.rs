//! Revolut Pay billing provider integration (Europe).
//!
//! Supports fast bank transfers, card payments, and subscriptions.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Revolut Pay billing provider.
pub struct RevolutProvider {
    pub api_key: String,
    pub webhook_secret: String,
    pub base_url: String,
}

impl RevolutProvider {
    pub fn new(api_key: String, webhook_secret: String) -> Self {
        Self {
            api_key,
            webhook_secret,
            base_url: "https://sandbox-b2b.revolut.com/api/1.0".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl BillingProvider for RevolutProvider {
    fn provider_name(&self) -> &'static str {
        "revolut"
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
            "amount": plan_slug.parse::<i64>().unwrap_or(9999),
            "currency": "EUR",
            "description": format!("Plan: {}", plan_slug),
            "metadata": {
                "user_id": user_id.to_string(),
                "customer_email": customer_email,
                "plan_slug": plan_slug,
            },
            "redirect_url": success_url,
            "cancel_url": cancel_url,
        });

        let resp = client
            .post(format!("{}/orders", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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
            checkout_url: data["checkout_url"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/subscriptions/{}/cancel",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            "{}/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let current_end = data["current_period_end"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());

        Ok(SubscriptionInfo {
            provider_subscription_id: data["id"].as_str().unwrap_or_default().to_string(),
            status: data["state"].as_str().unwrap_or_default().to_string(),
            current_period_end: current_end,
            cancel_at_period_end: false,
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Revolut webhook verification: HMAC-SHA256 of payload
        let payload_str = String::from_utf8(event.payload.clone())
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let expected = {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            let mut mac =
                Hmac::<Sha256>::new_from_slice(self.webhook_secret.as_bytes()).expect("HMAC key");
            mac.update(event.payload.as_slice());
            let result = mac.finalize().into_bytes();
            hex::encode(result)
        };

        if !constant_time_eq_str(&expected, &event.signature) {
            return Err(BillingError::WebhookVerification(
                "Signature mismatch".to_string(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let event_type = data["event"].as_str().unwrap_or_default().to_string();

        Ok(ParsedWebhook {
            event_type,
            customer_id: data["order"]["customer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: data["subscription"]["id"].as_str().map(String::from),
            payment_id: data["order"]["id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // Revolut doesn't have a native billing portal
        Ok(format!(
            "https://business.revolut.com/customer/{}?return_url={}",
            provider_customer_id,
            urlencoding::encode(return_url)
        ))
    }
}

fn constant_time_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revolut_provider_name() {
        let provider = RevolutProvider::new("api_key".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "revolut");
    }

    #[test]
    fn test_revolut_new() {
        let provider = RevolutProvider::new("rev_test_key".into(), "whsec_test".into());
        assert_eq!(provider.api_key, "rev_test_key");
        assert_eq!(provider.webhook_secret, "whsec_test");
        assert_eq!(provider.base_url, "https://sandbox-b2b.revolut.com/api/1.0");
    }

    #[test]
    fn test_revolut_custom_base_url() {
        let provider = RevolutProvider::new("k".into(), "w".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }
}
