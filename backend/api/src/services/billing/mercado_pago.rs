//! Mercado Pago billing provider integration (LATAM).
//!
//! Supports PIX, OXXO, credit cards, and Checkout Pro.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Mercado Pago billing provider.
pub struct MercadoPagoProvider {
    pub access_token: String,
    pub webhook_secret: String,
    pub base_url: String,
}

impl MercadoPagoProvider {
    pub fn new(access_token: String, webhook_secret: String) -> Self {
        Self {
            access_token,
            webhook_secret,
            base_url: "https://api.mercadopago.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl BillingProvider for MercadoPagoProvider {
    fn provider_name(&self) -> &'static str {
        "mercado_pago"
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
            "items": [
                {
                    "title": format!("Plan: {}", plan_slug),
                    "quantity": 1,
                    "unit_price": plan_slug.parse::<f64>().unwrap_or(99.90),
                    "currency_id": "BRL",
                }
            ],
            "payer": {
                "email": customer_email,
            },
            "back_urls": {
                "success": success_url,
                "failure": cancel_url,
                "pending": success_url,
            },
            "auto_return": "approved",
            "external_reference": user_id.to_string(),
            "notification_url": success_url,
        });

        let resp = client
            .post(format!("{}/checkout/preferences", self.base_url))
            .header("Authorization", format!("Bearer {}", self.access_token))
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
            checkout_url: data["init_point"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        let client = reqwest::Client::new();
        let url = format!("{}/preapproval/{}", self.base_url, provider_subscription_id);

        let resp = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "status": "cancelled" }))
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
        let url = format!("{}/preapproval/{}", self.base_url, provider_subscription_id);

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
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

        let current_end = data["next_payment_date"]
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
        // Mercado Pago uses x-signature header: ts=<timestamp>,v1=<hmac-sha256>
        let payload_str = String::from_utf8(event.payload.clone())
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Parse the signature: "ts=<timestamp>,v1=<hash>"
        let signature = &event.signature;
        let parts: std::collections::HashMap<&str, &str> = signature
            .split(',')
            .filter_map(|part| {
                let mut kv = part.splitn(2, '=');
                let k = kv.next()?;
                let v = kv.next()?;
                Some((k, v))
            })
            .collect();

        let ts = parts.get("ts").unwrap_or(&"");
        let v1 = parts.get("v1").unwrap_or(&"");

        // Verify: HMAC-SHA256(ts + payload, webhook_secret)
        let manifest = format!("{}{}", ts, payload_str);
        let expected = {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            let mut mac =
                Hmac::<Sha256>::new_from_slice(self.webhook_secret.as_bytes()).expect("HMAC key");
            mac.update(manifest.as_bytes());
            let result = mac.finalize().into_bytes();
            hex::encode(result)
        };

        if !constant_time_eq_str(&expected, v1) {
            return Err(BillingError::WebhookVerification(
                "Signature mismatch".to_string(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let event_type = data["type"].as_str().unwrap_or_default().to_string();

        Ok(ParsedWebhook {
            event_type,
            customer_id: data["data"]["payer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: data["data"]["preapproval_id"].as_str().map(String::from),
            payment_id: data["data"]["id"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // Mercado Pago doesn't have a native portal — redirect to customer management
        Ok(format!(
            "https://www.mercadopago.com.br/subscriptions#c/{}/{}",
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
    fn test_mercado_pago_provider_name() {
        let provider = MercadoPagoProvider::new("token".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "mercado_pago");
    }

    #[test]
    fn test_mercado_pago_new() {
        let provider = MercadoPagoProvider::new("APP_USR-abc123".into(), "whsec_def".into());
        assert_eq!(provider.access_token, "APP_USR-abc123");
        assert_eq!(provider.webhook_secret, "whsec_def");
        assert_eq!(provider.base_url, "https://api.mercadopago.com");
    }

    #[test]
    fn test_mercado_pago_custom_base_url() {
        let provider = MercadoPagoProvider::new("token".into(), "wh".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }
}
