//! Polar.sh billing provider integration.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

/// Polar.sh billing provider.
pub struct PolarProvider {
    pub access_token: String,
    pub webhook_secret: String,
    pub base_url: String,
}

impl PolarProvider {
    pub fn new(access_token: String, webhook_secret: String) -> Self {
        Self {
            access_token,
            webhook_secret,
            // Production by default; override with the sandbox host via
            // POLAR_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("POLAR_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.polar.sh".to_string()),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Best-effort fetch of a subscription's `current_period_end` (RFC 3339).
    /// Used on a succeeded `checkout.*` event (whose object is the checkout, not
    /// the subscription, and so carries no period end) to obtain the linked
    /// subscription's authoritative end so the paywall can admit the paying
    /// subscriber. Returns `None` on any failure → fail-closed (audit F#11
    /// round-2).
    async fn fetch_subscription_period_end(&self, subscription_id: &str) -> Option<i64> {
        let client = reqwest::Client::new();
        let url = format!("{}/v1/subscriptions/{}", self.base_url, subscription_id);
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let data: serde_json::Value = resp.json().await.ok()?;
        super::provider::period_end_to_unix(data.get("current_period_end"))
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
            .post(format!("{}/v1/checkouts/", self.base_url))
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
            "{}/v1/subscriptions/{}/cancel",
            self.base_url, provider_subscription_id
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
            "{}/v1/subscriptions/{}",
            self.base_url, provider_subscription_id
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

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Polar.sh signs the raw body with HMAC-SHA256(webhook_secret, body) and
        // sends the hex digest in X-Polar-Signature. Previously this method
        // performed NO verification at all — any forged JSON was accepted and
        // the webhook_secret field was dead.
        let sig = super::webhook_util::header_str(&event.headers, "X-Polar-Signature")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing X-Polar-Signature header".into())
            })?;
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.as_bytes(),
            &event.payload,
            &sig,
        ) {
            return Err(BillingError::WebhookVerification(
                "Polar signature mismatch".into(),
            ));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        let data: serde_json::Value = serde_json::from_str(payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Normalize Polar's native event taxonomy to the canonical vocabulary
        // the dispatch matches on (audit F#11 residual). Polar fires
        // `checkout.updated` on ANY status transition, so only map it to
        // CHECKOUT_COMPLETED when the checkout actually succeeded — otherwise a
        // pre-payment `open`/`confirmed` update would consume the single-use
        // intent and grant prematurely.
        let native_event = data["type"].as_str().unwrap_or_default();
        let checkout_succeeded = data["data"]["status"].as_str() == Some("succeeded");
        let event_type = match native_event {
            "checkout.updated" | "checkout.created" if checkout_succeeded => {
                super::provider::canonical::CHECKOUT_COMPLETED
            }
            "subscription.updated" | "subscription.active" => {
                super::provider::canonical::SUBSCRIPTION_UPDATED
            }
            "subscription.revoked" | "subscription.canceled" => {
                super::provider::canonical::SUBSCRIPTION_DELETED
            }
            other => other,
        }
        .to_string();

        let obj = &data["data"];
        // Resolve the billing period end. `subscription.*` lifecycle events carry
        // it inline (RFC 3339); a succeeded `checkout.*` event's object is the
        // checkout (no period end), so fetch the linked subscription's
        // authoritative end. Without a real value the paywall fails closed
        // (audit F#11 round-2); fetch failures degrade to None.
        let current_period_end = match super::provider::period_end_to_unix(obj.get("current_period_end")) {
            Some(ts) => Some(ts),
            None => match obj.get("subscription_id").and_then(|v| v.as_str()) {
                Some(sub_id) => self.fetch_subscription_period_end(sub_id).await,
                None => None,
            },
        };

        Ok(ParsedWebhook {
            event_type,
            customer_id: obj["customer_id"].as_str().unwrap_or_default().to_string(),
            subscription_id: obj["subscription_id"]
                .as_str()
                .or_else(|| obj["id"].as_str())
                .map(String::from),
            payment_id: obj["order_id"].as_str().map(String::from),
            // Polar keys the checkout intent by the checkout id (its
            // `create_checkout` returns it as `session_id`); the checkout
            // webhook echoes it back as `data.id`.
            checkout_session_id: obj["id"].as_str().map(String::from),
            current_period_end,
            subscription_status: obj["status"].as_str().map(String::from),
            user_id: obj["metadata"]["user_id"]
                .as_str()
                .or_else(|| obj["user_id"].as_str())
                .and_then(|s| s.parse().ok()),
            amount_cents: obj["total_amount"].as_i64(),
            currency: obj["currency"].as_str().map(String::from),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polar_provider_name() {
        let provider = PolarProvider::new("polar_token".into(), "polar_secret".into());
        assert_eq!(provider.provider_name(), "polar");
    }

    #[test]
    fn test_polar_new() {
        let provider = PolarProvider::new("access_tok_abc".into(), "whsec_xyz".into());
        assert_eq!(provider.access_token, "access_tok_abc");
        assert_eq!(provider.webhook_secret, "whsec_xyz");
    }

    use crate::services::billing::webhook_util;

    /// Sign a Polar webhook: HMAC-SHA256(secret, raw body) in
    /// `X-Polar-Signature` (no timestamp).
    fn signed_polar(payload: &[u8], secret: &str) -> WebhookEvent {
        let sig = webhook_util::hmac_sha256_hex(secret.as_bytes(), payload);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("X-Polar-Signature", sig.parse().unwrap());
        WebhookEvent {
            provider: "polar".into(),
            payload: payload.to_vec(),
            headers,
        }
    }

    /// Native Polar events must normalize to the canonical vocabulary the
    /// provider-agnostic dispatch matches on (audit F#11). Critically,
    /// `checkout.updated` fires on ANY status transition, so it must only map to
    /// CHECKOUT_COMPLETED when `status == "succeeded"` — otherwise a pre-payment
    /// `open` update would consume the single-use intent and grant prematurely.
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let provider = PolarProvider::new("tok".into(), "whsec".into());

        let cases: &[(&str, &str)] = &[
            (
                r#"{"type":"checkout.updated","data":{"id":"co_1","status":"succeeded","customer_id":"cus_1","subscription_id":"sub_1","metadata":{"user_id":"42"},"total_amount":9999,"currency":"usd"}}"#,
                "checkout.session.completed",
            ),
            (
                r#"{"type":"checkout.created","data":{"id":"co_1","status":"succeeded"}}"#,
                "checkout.session.completed",
            ),
            (
                r#"{"type":"subscription.updated","data":{"id":"sub_1","status":"active"}}"#,
                "customer.subscription.updated",
            ),
            (
                r#"{"type":"subscription.active","data":{"id":"sub_1","status":"active"}}"#,
                "customer.subscription.updated",
            ),
            (
                r#"{"type":"subscription.revoked","data":{"id":"sub_1","status":"canceled"}}"#,
                "customer.subscription.deleted",
            ),
            (
                r#"{"type":"subscription.canceled","data":{"id":"sub_1","status":"canceled"}}"#,
                "customer.subscription.deleted",
            ),
        ];
        for (body, expected) in cases {
            let evt = signed_polar(body.as_bytes(), "whsec");
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // GATING: a checkout.updated that has NOT yet succeeded must NOT map to
        // CHECKOUT_COMPLETED (would prematurely consume the single-use intent).
        let evt = signed_polar(
            br#"{"type":"checkout.updated","data":{"id":"co_1","status":"open"}}"#,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.event_type, "checkout.updated");

        // Structured fields on a succeeded checkout.
        let evt = signed_polar(
            br#"{"type":"checkout.updated","data":{"id":"co_1","status":"succeeded","customer_id":"cus_1","subscription_id":"sub_1","metadata":{"user_id":"42"},"total_amount":9999,"currency":"usd"}}"#,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("co_1"));
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_1"));
        assert_eq!(parsed.subscription_status.as_deref(), Some("succeeded"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(9999));
        assert_eq!(parsed.currency.as_deref(), Some("usd"));
    }
}
