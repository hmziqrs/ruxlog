//! Airwallex billing provider integration (Global/APAC).
//!
//! Supports multi-currency, cross-border payments, and subscriptions.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Airwallex billing provider.
pub struct AirwallexProvider {
    pub client_id: String,
    pub api_key: String,
    pub webhook_secret: String,
    pub base_url: String,
}

impl AirwallexProvider {
    pub fn new(client_id: String, api_key: String, webhook_secret: String) -> Self {
        Self {
            client_id,
            api_key,
            webhook_secret,
            base_url: "https://api-demo.airwallex.com/api/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

impl AirwallexProvider {
    /// Authenticate and get a bearer token from Airwallex.
    async fn get_access_token(&self) -> Result<String, BillingError> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/authentication/login", self.base_url))
            .header("x-client-id", &self.client_id)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(BillingError::ProviderApi(format!("Auth failed: {}", body)));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        data["token"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| BillingError::ProviderApi("No token in auth response".to_string()))
    }
}

#[async_trait]
impl BillingProvider for AirwallexProvider {
    fn provider_name(&self) -> &'static str {
        "airwallex"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();

        // Create a PaymentIntent first
        let body = serde_json::json!({
            "amount": plan_slug.parse::<f64>().unwrap_or(99.00),
            "currency": "USD",
            "merchant_order_id": format!("order_{}_{}", user_id, chrono::Utc::now().timestamp()),
            "metadata": {
                "user_id": user_id.to_string(),
                "plan_slug": plan_slug,
                "customer_email": customer_email,
            },
            "return_url": success_url,
            "cancel_url": cancel_url,
        });

        let resp = client
            .post(format!("{}/pa/payment_intents/create", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
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

        let client_secret = data["client_secret"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        // Generate checkout URL using Airwallex hosted checkout
        let checkout_url = format!(
            "https://demo.airwallex.com/checkout?intent_id={}&client_secret={}",
            data["id"].as_str().unwrap_or_default(),
            urlencoding::encode(&client_secret),
        );

        Ok(CheckoutSession {
            session_id: data["id"].as_str().unwrap_or_default().to_string(),
            checkout_url,
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = format!(
            "{}/pa/subscriptions/{}/cancel",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
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
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = format!(
            "{}/pa/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
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
            status: data["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: current_end,
            cancel_at_period_end: data["cancel_at_period_end"].as_bool().unwrap_or(false),
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Airwallex webhook verification: HMAC-SHA256 of payload
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

        let event_type = data["event_type"].as_str().unwrap_or_default().to_string();

        let obj = &data["data"]["entity"];

        Ok(ParsedWebhook {
            event_type,
            customer_id: obj["customer_id"].as_str().unwrap_or_default().to_string(),
            subscription_id: obj["subscription_id"].as_str().map(String::from),
            payment_id: obj["payment_intent_id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // Airwallex doesn't provide a native billing portal
        Ok(format!(
            "https://demo.airwallex.com/customer/{}?return_url={}",
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
    fn test_airwallex_provider_name() {
        let provider = AirwallexProvider::new("client_id".into(), "api_key".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "airwallex");
    }

    #[test]
    fn test_airwallex_new() {
        let provider = AirwallexProvider::new(
            "awx_test_client".into(),
            "awx_test_key".into(),
            "whsec_test".into(),
        );
        assert_eq!(provider.client_id, "awx_test_client");
        assert_eq!(provider.api_key, "awx_test_key");
        assert_eq!(provider.webhook_secret, "whsec_test");
    }

    #[test]
    fn test_airwallex_custom_base_url() {
        let provider = AirwallexProvider::new("c".into(), "k".into(), "w".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }
}
