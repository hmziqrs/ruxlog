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
    pub base_url: String,
}

impl LemonSqueezyProvider {
    /// Create a new provider from explicit values.
    pub fn new(api_key: String, webhook_secret: String, store_id: String) -> Self {
        Self {
            api_key,
            webhook_secret,
            store_id,
            // Production by default; override with the sandbox host via
            // LEMONSQUEEZY_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("LEMONSQUEEZY_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.lemonsqueezy.com".to_string()),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Create a new provider from environment variables.
    pub fn from_env() -> Result<Self, BillingError> {
        let api_key = std::env::var("LEMONSQUEEZY_API_KEY")
            .map_err(|_| BillingError::Config("LEMONSQUEEZY_API_KEY not set".to_string()))?;
        let webhook_secret = std::env::var("LEMONSQUEEZY_WEBHOOK_SECRET")
            .map_err(|_| BillingError::Config("LEMONSQUEEZY_WEBHOOK_SECRET not set".to_string()))?;
        let store_id = std::env::var("LEMONSQUEEZY_STORE_ID")
            .map_err(|_| BillingError::Config("LEMONSQUEEZY_STORE_ID not set".to_string()))?;
        Ok(Self::new(api_key, webhook_secret, store_id))
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
            .post(format!("{}/v1/checkouts", self.base_url))
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
            "{}/v1/subscriptions/{}",
            self.base_url, provider_subscription_id
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
            "{}/v1/subscriptions/{}",
            self.base_url, provider_subscription_id
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
            provider_subscription_id: data["data"]["id"].as_str().unwrap_or_default().to_string(),
            status: attrs["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: attrs["renews_at"].as_str().and_then(|s| s.parse().ok()),
            cancel_at_period_end: attrs["cancelled"].as_bool().unwrap_or(false),
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // LemonSqueezy signs the raw body with HMAC-SHA256(webhook_secret, body)
        // and sends the hex digest in X-Signature. No timestamp, no freshness.
        let sig = super::webhook_util::header_str(&event.headers, "X-Signature")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing X-Signature header".into())
            })?;
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.as_bytes(),
            &event.payload,
            &sig,
        ) {
            return Err(BillingError::WebhookVerification(
                "LemonSqueezy signature mismatch".into(),
            ));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let obj = &data["data"]["attributes"];

        // Normalize LemonSqueezy's native event taxonomy to the canonical
        // vocabulary the dispatch matches on (audit F#11 residual). An
        // `order_created` (one-time purchase) and `subscription_created` (first
        // activation) are both the checkout-completion signal; renewals and
        // cancellations map to the subscription lifecycle events.
        //
        // NOTE on id round-trip: LS keys its checkout intent by the checkout id
        // (its `create_checkout` returns it as `session_id`), but the webhook
        // resource id (`data.id`) is the ORDER/SUBSCRIPTION id, not the original
        // checkout id. So the checkout arm's intent recovery will miss and the
        // dispatch refuses to grant (fail-closed, audit F#2/F#10). Routing to
        // the correct arm (instead of the old silent `_ =>` drop) is the fix;
        // correlating the checkout id back through an LS API fetch is the
        // accepted deferred enhancement. Subscription UPDATE/DELETE arms find
        // rows by `provider_subscription_id` and DO process normally.
        let native_event = data["meta"]["event_name"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "order_created" | "subscription_created" => {
                super::provider::canonical::CHECKOUT_COMPLETED
            }
            "subscription_updated" => super::provider::canonical::SUBSCRIPTION_UPDATED,
            "subscription_cancelled" | "subscription_expired" => {
                super::provider::canonical::SUBSCRIPTION_DELETED
            }
            "subscription_payment_success" => super::provider::canonical::PAYMENT_SUCCEEDED,
            // `order_refunded` is NOT a successful payment — do not map it to
            // PAYMENT_SUCCEEDED (audit F#11 residual: a refund would be recorded
            // as a new succeeded payment). Let it fall through to the dispatch's
            // log-only unhandled arm. (A dedicated refund/revocation handler can
            // be wired later.)
            other => other,
        }
        .to_string();

        Ok(ParsedWebhook {
            event_type,
            customer_id: obj["customer_id"].as_str().unwrap_or_default().to_string(),
            subscription_id: data["data"]["id"].as_str().map(String::from),
            payment_id: obj["order_id"].as_str().map(String::from),
            // Best-effort (see NOTE): echoes the order/sub id, not the stored
            // checkout id. Populated so a future correlation enhancement can
            // recover the intent without touching this parsing.
            checkout_session_id: data["data"]["id"].as_str().map(String::from),
            // LemonSqueezy subscription attributes expose `renews_at` (RFC 3339).
            current_period_end: super::provider::period_end_to_unix(obj.get("renews_at")),
            subscription_status: obj["status"].as_str().map(String::from),
            user_id: obj["custom_data"]["user_id"]
                .as_str()
                .or_else(|| data["meta"]["custom_data"]["user_id"].as_str())
                .and_then(|s| s.parse().ok()),
            // LS order totals are integer minor units (may arrive as string).
            amount_cents: obj["total"]
                .as_i64()
                .or_else(|| obj["total"].as_str().and_then(|s| s.parse().ok()))
                .or_else(|| obj["total_paid"].as_i64()),
            currency: obj["currency"]
                .as_str()
                .or_else(|| obj["currency_code"].as_str())
                .map(String::from),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lemon_squeezy_provider_name() {
        let provider =
            LemonSqueezyProvider::new("api_key".into(), "whsec".into(), "store_1".into());
        assert_eq!(provider.provider_name(), "lemon_squeezy");
    }

    #[test]
    fn test_lemon_squeezy_new() {
        let provider =
            LemonSqueezyProvider::new("key_abc".into(), "secret_def".into(), "store_123".into());
        assert_eq!(provider.api_key, "key_abc");
        assert_eq!(provider.webhook_secret, "secret_def");
        assert_eq!(provider.store_id, "store_123");
    }

    #[test]
    fn test_lemon_squeezy_from_env_missing() {
        // Ensure none of the required env vars are set
        std::env::remove_var("LEMONSQUEEZY_API_KEY");
        std::env::remove_var("LEMONSQUEEZY_WEBHOOK_SECRET");
        std::env::remove_var("LEMONSQUEEZY_STORE_ID");
        let result = LemonSqueezyProvider::from_env();
        assert!(result.is_err());
    }

    use crate::services::billing::webhook_util;

    /// Sign a LemonSqueezy webhook: HMAC-SHA256(secret, raw body) in
    /// `X-Signature` (no timestamp).
    fn signed_ls(payload: &[u8], secret: &str) -> WebhookEvent {
        let sig = webhook_util::hmac_sha256_hex(secret.as_bytes(), payload);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("X-Signature", sig.parse().unwrap());
        WebhookEvent {
            provider: "lemon_squeezy".into(),
            payload: payload.to_vec(),
            headers,
        }
    }

    /// Native LemonSqueezy events must normalize to the canonical vocabulary
    /// the provider-agnostic dispatch matches on (audit F#11). NOTE: the LS
    /// resource id does not round-trip to the stored checkout id, so the
    /// checkout arm fails closed on intent recovery — but the event still
    /// reaches the correct arm (not the old silent `_ =>` drop).
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let provider =
            LemonSqueezyProvider::new("k".into(), "whsec".into(), "store_1".into());

        let mk = |event: &str| -> Vec<u8> {
            const TPL: &str = r#"{"meta":{"event_name":"__EV__"},"data":{"id":"ord_1","attributes":{"status":"active","customer_id":"ctm_1","custom_data":{"user_id":"42"},"total":1000,"currency":"USD","order_id":"ord_1","renews_at":"2026-12-31T00:00:00Z"}}}"#;
            TPL.replace("__EV__", event).into_bytes()
        };

        let cases: &[(&str, &str)] = &[
            ("order_created", "checkout.session.completed"),
            ("subscription_created", "checkout.session.completed"),
            ("subscription_updated", "customer.subscription.updated"),
            ("subscription_cancelled", "customer.subscription.deleted"),
            ("subscription_expired", "customer.subscription.deleted"),
            ("subscription_payment_success", "invoice.payment_succeeded"),
            // A refund is NOT a succeeded payment — passes through (audit F#11).
            ("order_refunded", "order_refunded"),
            // Unmapped → passthrough.
            ("license_key_created", "license_key_created"),
        ];
        for (event, expected) in cases {
            let body = mk(event);
            let evt = signed_ls(&body, "whsec");
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "event={event}");
        }

        // Structured fields.
        let evt = signed_ls(&mk("subscription_updated"), "whsec");
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.subscription_status.as_deref(), Some("active"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(1000));
        assert_eq!(parsed.currency.as_deref(), Some("USD"));
    }
}
