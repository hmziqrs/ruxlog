//! Razorpay billing provider integration (India/APAC).
//!
//! Supports UPI, net banking, cards, wallets, and subscriptions.

use async_trait::async_trait;
use base64::Engine;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Razorpay billing provider.
pub struct RazorpayProvider {
    pub key_id: String,
    pub key_secret: String,
    pub webhook_secret: String,
    pub base_url: String,
}

impl RazorpayProvider {
    pub fn new(key_id: String, key_secret: String, webhook_secret: String) -> Self {
        Self {
            key_id,
            key_secret,
            webhook_secret,
            base_url: "https://api.razorpay.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl BillingProvider for RazorpayProvider {
    fn provider_name(&self) -> &'static str {
        "razorpay"
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
        let receipt = format!("rcpt_{}_{}", user_id, chrono::Utc::now().timestamp());

        let body = serde_json::json!({
            "type": "link",
            "amount": plan_slug.parse::<i64>().unwrap_or(99900),
            "currency": "INR",
            "description": format!("Subscription for user {}", user_id),
            "customer": {
                "email": customer_email,
            },
            "notify": {
                "sms": false,
                "email": true,
            },
            "callback_url": success_url,
            "callback_method": "get",
            "receipt": receipt,
            "notes": {
                "user_id": user_id.to_string(),
                "plan_slug": plan_slug,
            },
        });

        let resp = client
            .post(format!("{}/payment_links", self.base_url))
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD
                        .encode(format!("{}:{}", self.key_id, self.key_secret))
                ),
            )
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
            checkout_url: data["short_url"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let client = reqwest::Client::new();

        if immediately {
            let url = format!(
                "{}/subscriptions/{}/cancel",
                self.base_url, provider_subscription_id
            );
            let resp = client
                .post(&url)
                .header(
                    "Authorization",
                    format!(
                        "Basic {}",
                        base64::engine::general_purpose::STANDARD
                            .encode(format!("{}:{}", self.key_id, self.key_secret))
                    ),
                )
                .json(&serde_json::json!({ "cancel_at_cycle_end": 0 }))
                .send()
                .await
                .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(BillingError::ProviderApi(body));
            }
        } else {
            let url = format!(
                "{}/subscriptions/{}/cancel",
                self.base_url, provider_subscription_id
            );
            let resp = client
                .post(&url)
                .header(
                    "Authorization",
                    format!(
                        "Basic {}",
                        base64::engine::general_purpose::STANDARD
                            .encode(format!("{}:{}", self.key_id, self.key_secret))
                    ),
                )
                .json(&serde_json::json!({ "cancel_at_cycle_end": 1 }))
                .send()
                .await
                .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(BillingError::ProviderApi(body));
            }
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
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD
                        .encode(format!("{}:{}", self.key_id, self.key_secret))
                ),
            )
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

        let current_end = data["current_end"]
            .as_i64()
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.fixed_offset());

        Ok(SubscriptionInfo {
            provider_subscription_id: data["id"].as_str().unwrap_or_default().to_string(),
            status: data["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: current_end,
            cancel_at_period_end: false,
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Razorpay webhook verification: HMAC-SHA256 of payload with webhook secret
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

        let payload_obj = &data["payload"]["payment"]["entity"];
        let sub_obj = &data["payload"]["subscription"]["entity"];

        Ok(ParsedWebhook {
            event_type,
            customer_id: payload_obj["customer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: sub_obj["id"].as_str().map(String::from),
            payment_id: payload_obj["id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // Razorpay doesn't have a native billing portal — return a customer-facing URL
        Ok(format!(
            "https://dashboard.razorpay.com/app/customers/{}?return_url={}",
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
    fn test_razorpay_provider_name() {
        let provider = RazorpayProvider::new("key_id".into(), "key_secret".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "razorpay");
    }

    #[test]
    fn test_razorpay_new() {
        let provider =
            RazorpayProvider::new("rzp_test_abc".into(), "secret123".into(), "whsec456".into());
        assert_eq!(provider.key_id, "rzp_test_abc");
        assert_eq!(provider.key_secret, "secret123");
        assert_eq!(provider.webhook_secret, "whsec456");
        assert_eq!(provider.base_url, "https://api.razorpay.com/v1");
    }

    #[test]
    fn test_razorpay_custom_base_url() {
        let provider = RazorpayProvider::new("key".into(), "secret".into(), "wh".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq_str("abc", "abc"));
        assert!(!constant_time_eq_str("abc", "abd"));
        assert!(!constant_time_eq_str("abc", "abcd"));
    }
}
