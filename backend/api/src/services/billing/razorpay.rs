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
            // Production by default; override with the sandbox host via
            // RAZORPAY_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("RAZORPAY_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.razorpay.com/v1".to_string()),
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
        // Razorpay subscription creation has no per-subscription success URL —
        // the hosted `short_url` auth page is the entry point and the verified
        // `subscription.activated` webhook (not a redirect) drives the grant.
        _success_url: &str,
        _cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let client = reqwest::Client::new();

        // Create a REAL Razorpay subscription (not a one-time payment link) so
        // the `session_id` we store — and key the checkout intent by — is a
        // subscription id (`sub_…`). That is exactly the id
        // `subscription.activated` echoes back in `payload.subscription.entity`,
        // so the checkout arm's intent recovery connects (audit F#11 residual;
        // the prior payment-link flow keyed the intent on `plink_…` while the
        // webhook emitted `sub_…`, so every subscription checkout silently never
        // granted). A subscription also carries an authoritative `current_end`,
        // giving the grant a real period end so the paywall admits the paying
        // subscriber (it fails closed on a missing period end).
        //
        // `plan_slug` is the provider-side Razorpay plan id — the plan's amount
        // and billing cycle live in Razorpay, not here. `total_count` (number of
        // charge cycles) is mandatory; default 12, override per deployment.
        let total_count: i64 = std::env::var("RAZORPAY_SUBSCRIPTION_TOTAL_COUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(12);

        let body = serde_json::json!({
            "plan_id": plan_slug,
            "total_count": total_count,
            "quantity": 1,
            "customer_notify": 1,
            "notes": {
                "user_id": user_id.to_string(),
                "plan_slug": plan_slug,
                "email": customer_email,
            },
        });

        let resp = client
            .post(format!("{}/subscriptions", self.base_url))
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
        // Razorpay signs the raw body: HMAC-SHA256(webhook_secret, body), hex
        // digest in X-Razorpay-Signature. No timestamp is sent, so no freshness
        // check is possible. Verify over the raw bytes in constant time.
        let sig = super::webhook_util::header_str(&event.headers, "X-Razorpay-Signature")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing X-Razorpay-Signature header".into())
            })?;
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.as_bytes(),
            &event.payload,
            &sig,
        ) {
            return Err(BillingError::WebhookVerification(
                "Razorpay signature mismatch".into(),
            ));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Normalize Razorpay's native event taxonomy to the canonical vocabulary
        // the dispatch matches on (audit F#11 residual). `subscription.activated`
        // fires once the subscription starts (after first payment) and carries
        // the subscription id we keyed the intent by at create-checkout, so it
        // maps to CHECKOUT_COMPLETED; `subscription.charged` is a renewal.
        let native_event = data["event"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "subscription.activated" => super::provider::canonical::CHECKOUT_COMPLETED,
            "subscription.charged" => super::provider::canonical::SUBSCRIPTION_UPDATED,
            "subscription.cancelled" => super::provider::canonical::SUBSCRIPTION_DELETED,
            "payment.captured" | "payment.authorized" => {
                super::provider::canonical::PAYMENT_SUCCEEDED
            }
            other => other,
        }
        .to_string();

        let payload_obj = &data["payload"]["payment"]["entity"];
        let sub_obj = &data["payload"]["subscription"]["entity"];
        let sub_id = sub_obj["id"].as_str().map(String::from);

        Ok(ParsedWebhook {
            event_type,
            customer_id: payload_obj["customer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: sub_id.clone(),
            payment_id: payload_obj["id"].as_str().map(String::from),
            // Razorpay subscription entities expose `current_end` (Unix seconds).
            current_period_end: super::provider::period_end_to_unix(sub_obj.get("current_end")),
            // Razorpay keys the checkout intent by the subscription id (its
            // `create_checkout` returns the subscription id as `session_id`),
            // and `subscription.activated`'s entity id is that same id.
            checkout_session_id: sub_id,
            subscription_status: sub_obj["status"].as_str().map(String::from),
            user_id: sub_obj["notes"]["user_id"]
                .as_str()
                .or_else(|| payload_obj["notes"]["user_id"].as_str())
                .and_then(|s| s.parse().ok()),
            // Razorpay amounts are in paise (minor units).
            amount_cents: payload_obj["amount"]
                .as_i64()
                .or_else(|| payload_obj["amount_paid"].as_i64()),
            currency: payload_obj["currency"].as_str().map(String::from),
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

    use crate::services::billing::webhook_util;

    /// Sign a Razorpay webhook: HMAC-SHA256(secret, raw body) in
    /// `X-Razorpay-Signature` (no timestamp).
    fn signed_razorpay(payload: &[u8], secret: &str) -> WebhookEvent {
        let sig = webhook_util::hmac_sha256_hex(secret.as_bytes(), payload);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("X-Razorpay-Signature", sig.parse().unwrap());
        WebhookEvent {
            provider: "razorpay".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    /// Native Razorpay events must normalize to the canonical vocabulary the
    /// provider-agnostic dispatch matches on (audit F#11).
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let provider =
            RazorpayProvider::new("key".into(), "secret".into(), "whsec".into());

        let mk = |event: &str| -> Vec<u8> {
            // Build a minimal but well-formed Razorpay envelope.
            const TPL: &str = r#"{"event":"__EV__","payload":{"subscription":{"entity":{"id":"sub_1","status":"active","notes":{"user_id":"42"}}},"payment":{"entity":{"id":"pay_1","amount":99900,"currency":"INR"}}}}"#;
            TPL.replace("__EV__", event).into_bytes()
        };

        let cases: &[(&str, &str)] = &[
            ("subscription.activated", "checkout.session.completed"),
            ("subscription.charged", "customer.subscription.updated"),
            ("subscription.cancelled", "customer.subscription.deleted"),
            ("payment.captured", "invoice.payment_succeeded"),
        ];
        for (event, expected) in cases {
            let body = mk(event);
            let evt = signed_razorpay(&body, "whsec");
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "event={event}");
        }

        // Structured fields: subscription_id round-trips to the stored session
        // id (the checkout arm recovers the intent via the fallback chain).
        let evt = signed_razorpay(&mk("subscription.activated"), "whsec");
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_1"));
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("sub_1"));
        assert_eq!(parsed.subscription_status.as_deref(), Some("active"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(99900));
        assert_eq!(parsed.currency.as_deref(), Some("INR"));

        // An unmapped native event passes through (not silently dropped to a
        // canonical arm); the dispatch logs it as unhandled.
        let evt = signed_razorpay(&mk("payment.link.cancelled"), "whsec");
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.event_type, "payment.link.cancelled");
    }
}
