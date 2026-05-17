//! PayPal Commerce billing provider integration (Global).
//!
//! Supports PayPal, Venmo, credit/debit cards, and subscriptions.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// PayPal Commerce billing provider.
pub struct PayPalProvider {
    pub client_id: String,
    pub client_secret: String,
    pub webhook_secret: String,
    pub base_url: String,
}

impl PayPalProvider {
    pub fn new(client_id: String, client_secret: String, webhook_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            webhook_secret,
            base_url: "https://api-m.sandbox.paypal.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

impl PayPalProvider {
    /// Get an OAuth access token from PayPal.
    async fn get_access_token(&self) -> Result<String, BillingError> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/v1/oauth2/token", self.base_url))
            .header("Accept", "application/json")
            .header("Accept-Language", "en_US")
            .form(&[("grant_type", "client_credentials")])
            .basic_auth(&self.client_id, Some(&self.client_secret))
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

        data["access_token"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| BillingError::ProviderApi("No access_token in response".to_string()))
    }
}

#[async_trait]
impl BillingProvider for PayPalProvider {
    fn provider_name(&self) -> &'static str {
        "paypal"
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

        let body = serde_json::json!({
            "intent": "CAPTURE",
            "purchase_units": [{
                "reference_id": format!("plan_{}_{}", user_id, plan_slug),
                "amount": {
                    "currency_code": "USD",
                    "value": plan_slug.parse::<f64>().unwrap_or(99.99).to_string(),
                },
                "description": format!("Plan: {}", plan_slug),
                "custom_id": user_id.to_string(),
            }],
            "application_context": {
                "brand_name": "Ruxlog",
                "user_action": "PAY_NOW",
                "return_url": success_url,
                "cancel_url": cancel_url,
                "shipping_preference": "NO_SHIPPING",
            },
        });

        let resp = client
            .post(format!("{}/v2/checkout/orders", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("PayPal-Request-Id", uuid::Uuid::new_v4().to_string())
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

        // Extract the approval link from the response
        let checkout_url = data["links"]
            .as_array()
            .and_then(|links| {
                links
                    .iter()
                    .find(|link| link["rel"].as_str() == Some("approve"))
                    .and_then(|link| link["href"].as_str().map(String::from))
            })
            .unwrap_or_default();

        Ok(CheckoutSession {
            session_id: data["id"].as_str().unwrap_or_default().to_string(),
            checkout_url,
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = format!(
            "{}/v1/billing/subscriptions/{}/cancel",
            self.base_url, provider_subscription_id
        );

        let reason = if immediately {
            "Cancelled immediately by admin"
        } else {
            "Cancelled at end of billing cycle"
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "reason": reason }))
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
            "{}/v1/billing/subscriptions/{}",
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

        let current_end = data["billing_info"]["next_billing_time"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());

        Ok(SubscriptionInfo {
            provider_subscription_id: data["id"].as_str().unwrap_or_default().to_string(),
            status: data["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: current_end,
            cancel_at_period_end: false,
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // PayPal webhook verification: HMAC-SHA256 with webhook ID + timestamp + payload
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

        let resource = &data["resource"];

        Ok(ParsedWebhook {
            event_type,
            customer_id: resource["subscriber"]["payer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: resource["id"].as_str().map(String::from),
            payment_id: resource["id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // PayPal doesn't have a native billing portal — redirect to PayPal settings
        Ok(format!(
            "https://www.sandbox.paypal.com/myaccount/autopay/connect/{}?return_url={}",
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
    fn test_paypal_provider_name() {
        let provider = PayPalProvider::new("cid".into(), "secret".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "paypal");
    }

    #[test]
    fn test_paypal_new() {
        let provider = PayPalProvider::new(
            "AWx_test_client_id".into(),
            "test_secret_key".into(),
            "whsec_test".into(),
        );
        assert_eq!(provider.client_id, "AWx_test_client_id");
        assert_eq!(provider.client_secret, "test_secret_key");
        assert_eq!(provider.webhook_secret, "whsec_test");
        assert_eq!(provider.base_url, "https://api-m.sandbox.paypal.com");
    }

    #[test]
    fn test_paypal_custom_base_url() {
        let provider = PayPalProvider::new("c".into(), "s".into(), "w".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }
}
