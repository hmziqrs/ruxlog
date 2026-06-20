//! Stripe billing provider integration.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound Stripe call goes through this client (built once in
// `new` with timeouts, or overridden via `with_http_client` with the shared
// AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// Stripe billing provider.
pub struct StripeProvider {
    pub secret_key: String,
    pub webhook_secret: String,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl StripeProvider {
    pub fn new(secret_key: String, webhook_secret: String) -> Self {
        Self {
            secret_key,
            webhook_secret,
            // Production by default; override with the sandbox/test host via
            // STRIPE_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("STRIPE_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.stripe.com".to_string()),
            http_client: build_http_client(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// V-MED-10: inject the shared, timeout-configured client from `AppState`
    /// so this provider participates in connection pooling and never pins a
    /// handler thread on a hanging upstream.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// Best-effort fetch of a subscription's `current_period_end` (Unix seconds).
    /// Used on `checkout.session.completed`, whose object is the Checkout Session
    /// (which has no period end), to obtain the linked subscription's
    /// authoritative end so the paywall can admit the paying subscriber. Returns
    /// `None` on any failure (network, non-2xx, missing field) → fail-closed
    /// (audit F#11 round-2).
    async fn fetch_subscription_period_end(&self, subscription_id: &str) -> Option<i64> {
        let client = self.http_client.clone();
        let url = format!("{}/v1/subscriptions/{}", self.base_url, subscription_id);
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.secret_key))
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
impl BillingProvider for StripeProvider {
    fn provider_name(&self) -> &'static str {
        "stripe"
    }

    async fn create_checkout(
        &self,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let client = self.http_client.clone();
        let params = [
            ("mode", "subscription"),
            ("payment_method_types[0]", "card"),
            ("customer_email", customer_email),
            ("line_items[0][price]", plan_slug),
            ("line_items[0][quantity]", "1"),
            ("success_url", success_url),
            ("cancel_url", cancel_url),
            ("metadata[user_id]", &user_id.to_string()),
        ];

        let resp = client
            .post(format!("{}/v1/checkout/sessions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .form(&params)
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

    async fn create_post_checkout(
        &self,
        post_id: i32,
        amount_cents: i32,
        currency: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        // One-time payment (mode=payment) with an inline price_data so the
        // post's server-validated price is what's charged — never trusting a
        // client-supplied amount. metadata is echoed back in the webhook for
        // operator visibility, but the *grant* is driven by the server-bound
        // checkout intent (plan 1f/4e), not this metadata.
        let product_name = format!("Post #{}", post_id);
        let unit_amount = amount_cents.to_string();
        let user_id_s = user_id.to_string();
        let post_id_s = post_id.to_string();
        let params = [
            ("mode", "payment"),
            ("payment_method_types[0]", "card"),
            ("customer_email", customer_email),
            ("line_items[0][quantity]", "1"),
            ("line_items[0][price_data][currency]", currency),
            ("line_items[0][price_data][unit_amount]", &unit_amount),
            ("line_items[0][price_data][product_data][name]", &product_name),
            ("success_url", success_url),
            ("cancel_url", cancel_url),
            ("metadata[user_id]", &user_id_s),
            ("metadata[post_id]", &post_id_s),
        ];

        let client = self.http_client.clone();
        let resp = client
            .post(format!("{}/v1/checkout/sessions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .form(&params)
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
        immediately: bool,
    ) -> Result<(), BillingError> {
        let client = self.http_client.clone();
        let url = format!(
            "{}/v1/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let resp = if immediately {
            client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", self.secret_key))
                .send()
                .await
        } else {
            client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.secret_key))
                .form(&[("cancel_at_period_end", "true")])
                .send()
                .await
        }
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
        let client = self.http_client.clone();
        let url = format!(
            "{}/v1/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.secret_key))
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

        Ok(SubscriptionInfo {
            provider_subscription_id: data["id"].as_str().unwrap_or_default().to_string(),
            status: data["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: data["current_period_end"]
                .as_i64()
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.fixed_offset()),
            cancel_at_period_end: data["cancel_at_period_end"].as_bool().unwrap_or(false),
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Stripe-Signature: "t=<unix_seconds>,v1=<hex>". The tag is
        // HMAC-SHA256(secret, "{t}.{raw_body}") and is compared to v1.
        let sig_header = super::webhook_util::header_str(&event.headers, "Stripe-Signature")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing Stripe-Signature header".into())
            })?;

        let mut ts: Option<&str> = None;
        let mut v1: Option<&str> = None;
        for part in sig_header.split(',') {
            let mut kv = part.splitn(2, '=');
            match kv.next().map(str::trim) {
                Some("t") => ts = kv.next().map(str::trim),
                Some("v1") => v1 = kv.next().map(str::trim),
                _ => {}
            }
        }
        let ts_str = ts.ok_or_else(|| {
            BillingError::WebhookVerification("Stripe-Signature missing t=".into())
        })?;
        let v1 = v1.ok_or_else(|| {
            BillingError::WebhookVerification("Stripe-Signature missing v1=".into())
        })?;

        // Replay protection (CWE-294): reject timestamps outside the skew window.
        let ts_secs: i64 = ts_str.parse().map_err(|_| {
            BillingError::WebhookVerification("Stripe timestamp not an integer".into())
        })?;
        if !super::webhook_util::timestamp_fresh(ts_secs, chrono::Utc::now().timestamp()) {
            return Err(BillingError::WebhookVerification(format!(
                "Stripe timestamp outside tolerance (ts={ts_secs})"
            )));
        }

        // Sign "<t>.<raw_body>" and compare the v1 tag in constant time.
        let mut signed = Vec::with_capacity(ts_str.len() + 1 + event.payload.len());
        signed.extend_from_slice(ts_str.as_bytes());
        signed.push(b'.');
        signed.extend_from_slice(&event.payload);
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.as_bytes(),
            &signed,
            v1,
        ) {
            return Err(BillingError::WebhookVerification(
                "Stripe signature mismatch".into(),
            ));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        let data: serde_json::Value = serde_json::from_str(payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let event_type = data["type"].as_str().unwrap_or_default().to_string();
        let is_checkout = event_type == super::provider::canonical::CHECKOUT_COMPLETED;
        let obj = &data["data"]["object"];

        // Resolve the billing period end. `customer.subscription.*` events carry
        // it inline as a Unix-seconds integer; `checkout.session.completed` does
        // NOT (the object is the Checkout Session), so fetch the linked
        // subscription's authoritative `current_period_end`. Without a real value
        // the paywall fails closed (audit F#11 round-2); fetch failures degrade
        // to None. Only subscription checkouts carry a `subscription` to fetch —
        // one-time-payment (per-post) checkouts intentionally have None.
        let current_period_end = match super::provider::period_end_to_unix(obj.get("current_period_end")) {
            Some(ts) => Some(ts),
            None => match obj.get("subscription").and_then(|v| v.as_str()) {
                Some(sub_id) => self.fetch_subscription_period_end(sub_id).await,
                None => None,
            },
        };

        Ok(ParsedWebhook {
            // Stripe's event taxonomy IS the canonical vocabulary the dispatch
            // keys on, so it passes through unchanged.
            event_type,
            customer_id: obj["customer"].as_str().unwrap_or_default().to_string(),
            subscription_id: obj["subscription"].as_str().map(String::from),
            payment_id: obj["payment_intent"].as_str().map(String::from),
            current_period_end,
            // The checkout session id (cs_…) on `checkout.session.completed` is
            // exactly the `session_id` we stored the intent under at checkout.
            checkout_session_id: is_checkout
                .then(|| obj["id"].as_str().map(String::from))
                .flatten(),
            // Stripe status strings are already our canonical vocabulary.
            subscription_status: obj["status"].as_str().map(String::from),
            user_id: obj["metadata"]["user_id"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            amount_cents: obj["amount_total"]
                .as_i64()
                .or_else(|| obj["amount_paid"].as_i64()),
            currency: obj["currency"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        let client = self.http_client.clone();
        let params = [
            ("customer", provider_customer_id),
            ("return_url", return_url),
        ];

        let resp = client
            .post(format!("{}/v1/billing_portal/sessions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .form(&params)
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

        data["url"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| BillingError::ProviderApi("No portal URL in response".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::billing::webhook_util;

    #[test]
    fn test_stripe_provider_name() {
        let provider = StripeProvider::new("sk_test_key".into(), "whsec_secret".into());
        assert_eq!(provider.provider_name(), "stripe");
        assert_eq!(provider.base_url, "https://api.stripe.com");
    }

    #[test]
    fn test_stripe_new() {
        let provider = StripeProvider::new("sk_live_abc123".into(), "whsec_def456".into());
        assert_eq!(provider.secret_key, "sk_live_abc123");
        assert_eq!(provider.webhook_secret, "whsec_def456");
        assert_eq!(provider.base_url, "https://api.stripe.com");
    }

    #[test]
    fn test_stripe_custom_base_url() {
        let provider = StripeProvider::new("sk_test".into(), "whsec".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }

    /// Build a Stripe webhook event signed exactly as Stripe signs it:
    /// `Stripe-Signature: t=<ts>,v1=<HMAC-SHA256(secret, "{ts}.{body}")>`.
    fn signed_stripe_event(payload: &[u8], ts: i64, secret: &str) -> WebhookEvent {
        let ts_str = ts.to_string();
        let mut msg = Vec::with_capacity(ts_str.len() + 1 + payload.len());
        msg.extend_from_slice(ts_str.as_bytes());
        msg.push(b'.');
        msg.extend_from_slice(payload);
        let v1 = webhook_util::hmac_sha256_hex(secret.as_bytes(), &msg);

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "Stripe-Signature",
            format!("t={ts},v1={v1}").parse().unwrap(),
        );
        WebhookEvent {
            provider: "stripe".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    #[tokio::test]
    async fn verify_webhook_accepts_genuine_and_rejects_tampered() {
        let secret = "whsec_abc".to_string();
        let provider = StripeProvider::new("sk_test".into(), secret.clone());
        let body = br#"{"type":"invoice.payment_succeeded","data":{"object":{"customer":"cus_1","subscription":"sub_1","payment_intent":"pi_1"}}}"#;
        let now = chrono::Utc::now().timestamp();
        let genuine = signed_stripe_event(body, now, &secret);

        // Genuine signature → accepted, and the payment id is parsed out.
        let parsed = provider
            .verify_webhook(genuine.clone())
            .await
            .expect("genuine Stripe webhook must verify");
        assert_eq!(parsed.event_type, "invoice.payment_succeeded");
        assert_eq!(parsed.payment_id.as_deref(), Some("pi_1"));

        // Tampered body carrying the ORIGINAL signature → rejected.
        let mut tampered = genuine.clone();
        tampered.payload =
            br#"{"type":"invoice.payment_succeeded","data":{"object":{"payment_intent":"pi_EVIL"}}}"#
                .to_vec();
        assert!(provider.verify_webhook(tampered).await.is_err());

        // Missing Stripe-Signature header → rejected.
        let no_sig = WebhookEvent {
            provider: "stripe".into(),
            payload: body.to_vec(),
            headers: axum::http::HeaderMap::new(),
            query: None,
        };
        assert!(provider.verify_webhook(no_sig).await.is_err());

        // Stale timestamp (outside the replay window) → rejected even though the
        // signature is otherwise valid: re-sign at the stale ts to isolate the
        // freshness check from the signature check.
        let stale_ts = now - (webhook_util::MAX_SKEW_SECS + 60);
        let stale = signed_stripe_event(body, stale_ts, &secret);
        assert!(provider.verify_webhook(stale).await.is_err());
    }

    /// Stripe's event taxonomy IS the canonical vocabulary, so it passes through
    /// unchanged. The checkout session id on `checkout.session.completed` is the
    /// value the intent is keyed by (audit F#11 reference path).
    #[tokio::test]
    async fn verify_webhook_checkout_completed_round_trips_session_id() {
        let secret = "whsec_abc".to_string();
        let provider = StripeProvider::new("sk_test".into(), secret.clone());
        let body = br#"{"type":"checkout.session.completed","data":{"object":{"id":"cs_test_1","customer":"cus_1","subscription":"sub_1","status":"complete","metadata":{"user_id":"42"},"amount_total":9999,"currency":"usd"}}}"#;
        let now = chrono::Utc::now().timestamp();
        let evt = signed_stripe_event(body, now, &secret);

        let parsed = provider.verify_webhook(evt).await.expect("must verify");
        // Passthrough: Stripe's native type is already canonical.
        assert_eq!(parsed.event_type, "checkout.session.completed");
        // The checkout session id (cs_…) round-trips to the stored intent key.
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("cs_test_1"));
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_1"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(9999));
        assert_eq!(parsed.currency.as_deref(), Some("usd"));
    }
}
