//! Revolut Pay billing provider integration (Europe).
//!
//! Supports fast bank transfers, card payments, and subscriptions.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound Revolut call goes through this client (built once
// in `new` with timeouts, or overridden via `with_http_client` with the shared
// AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// Revolut Pay billing provider.
pub struct RevolutProvider {
    pub api_key: String,
    pub webhook_secret: String,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl RevolutProvider {
    pub fn new(api_key: String, webhook_secret: String) -> Self {
        Self {
            api_key,
            webhook_secret,
            // Production by default; override with the sandbox URL via
            // REVOLUT_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("REVOLUT_API_BASE_URL")
                .unwrap_or_else(|_| "https://merchant.revolut.com/api/1.0".to_string()),
            http_client: build_http_client(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// V-MED-10: inject the shared, timeout-configured client from `AppState`.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
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
        let client = self.http_client.clone();

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
        let client = self.http_client.clone();
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
        let client = self.http_client.clone();
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
        // Revolut Merchant API signs each webhook with HMAC-SHA256 and sends:
        //   - `Revolut-Signature: v1=<hex>`       (the hex MAC, prefixed `v1=`)
        //   - `Revolut-Request-Timestamp: <ts>`   (epoch millis)
        // The signed message is `<timestamp>.<raw_body>` (timestamp, a literal
        // dot, then the raw body) — official Revolut Merchant docs, "Verify the
        // payload signature". The previous code read a non-existent
        // `X-Revolut-Signature` header in a `<ts>.<hmac>` shape Revolut never
        // sends, and matched a fabricated `ORDER.COMPLETED` event against a
        // nested `order` object that does not exist — so every real Revolut
        // webhook was rejected at this gate (audit F#11 round-2).
        let sig_header =
            super::webhook_util::header_str(&event.headers, "Revolut-Signature").ok_or_else(|| {
                BillingError::WebhookVerification("Missing Revolut-Signature header".into())
            })?;
        // The header may list several `v1=` schemes comma-separated (revolut
        // rotates signing secrets); accept any that verifies. Strip the `v1=`
        // prefix from each candidate.
        let candidates: Vec<&str> = sig_header
            .split(',')
            .map(|c| c.trim())
            .filter_map(|c| c.strip_prefix("v1="))
            .collect();
        if candidates.is_empty() {
            return Err(BillingError::WebhookVerification(
                "Revolut-Signature has no 'v1=' scheme".into(),
            ));
        }
        let ts = super::webhook_util::header_str(&event.headers, "Revolut-Request-Timestamp")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing Revolut-Request-Timestamp header".into())
            })?;

        // Replay protection. Revolut sends epoch milliseconds; normalize to
        // seconds via the same magnitude heuristic as `period_end_to_unix`.
        let ts_raw: i64 = ts.trim().parse().map_err(|_| {
            BillingError::WebhookVerification("Revolut timestamp not an integer".into())
        })?;
        let ts_secs = if ts_raw > 1_000_000_000_000 { ts_raw / 1000 } else { ts_raw };
        if !super::webhook_util::timestamp_fresh(ts_secs, chrono::Utc::now().timestamp()) {
            return Err(BillingError::WebhookVerification(format!(
                "Revolut timestamp outside tolerance (ts={ts_secs})"
            )));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        // Signed message = "<timestamp>.<raw_body>".
        let mut manifest = Vec::with_capacity(ts.len() + 1 + event.payload.len());
        manifest.extend_from_slice(ts.as_bytes());
        manifest.push(b'.');
        manifest.extend_from_slice(&event.payload);
        // Accept the first candidate that matches a configured signing secret.
        // Ruxlog configures one secret, so there is effectively one candidate;
        // the loop is a no-op for normal traffic and harmless for rotation.
        let verified = candidates
            .iter()
            .any(|mac_hex| {
                super::webhook_util::verify_hmac_sha256_hex(
                    self.webhook_secret.as_bytes(),
                    &manifest,
                    mac_hex,
                )
            });
        if !verified {
            return Err(BillingError::WebhookVerification(
                "Revolut signature mismatch".into(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Normalize Revolut's native event taxonomy to the canonical vocabulary
        // the dispatch matches on. Revolut Merchant fires `ORDER_COMPLETED`
        // (underscore — the dotted `ORDER.COMPLETED` the previous code matched is
        // not a real event) when an order is paid; that is the grant signal and
        // carries the order id we keyed the intent by. Failure/declined events
        // are left unmapped and fall through to the dispatch's log-only arm
        // (never a grant).
        let native_event = data["event"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "ORDER_COMPLETED" => super::provider::canonical::CHECKOUT_COMPLETED,
            other => other,
        }
        .to_string();

        // The Revolut webhook payload is FLAT — { event, order_id,
        // merchant_order_ext_ref } — so customer/amount/currency/period fields
        // are NOT available here (they live on the Order resource via
        // GET /orders/{id}). The grant uses the server-bound checkout intent for
        // user_id/amount (the dispatch never trusts webhook JSON for granting),
        // keyed by the order id Revolut returned from `create_checkout`.
        let order_id = data["order_id"].as_str().map(String::from);

        Ok(ParsedWebhook {
            event_type,
            // Not present in the flat webhook; diagnostic only.
            customer_id: String::new(),
            // Revolut Merchant is a one-time order flow (POST /orders), not
            // recurring subscriptions — there is no subscription id on the
            // event. (A subscription arm therefore never fires for Revolut,
            // which is correct.)
            subscription_id: None,
            payment_id: order_id.clone(),
            // Revolut keys the checkout intent by the order id (its
            // `create_checkout` returns it as `session_id`); the webhook echoes
            // it back as the top-level `order_id`.
            checkout_session_id: order_id,
            // One-time order flow — no billing period. Revolut grants via the
            // per-post-purchase path, not a subscription row, so the absence of a
            // period end is correct (the paywall's fail-closed-on-None rule only
            // applies to the subscription path).
            current_period_end: None,
            subscription_status: None,
            // Not present in the flat webhook; the dispatch recovers user_id from
            // the server-bound checkout intent.
            user_id: None,
            amount_cents: None,
            currency: None,
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
        assert_eq!(provider.base_url, "https://merchant.revolut.com/api/1.0");
    }

    #[test]
    fn test_revolut_custom_base_url() {
        let provider = RevolutProvider::new("k".into(), "w".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }

    use crate::services::billing::webhook_util;

    /// Sign a Revolut webhook exactly as Revolut does: HMAC-SHA256(secret,
    /// "<ts>.<body>"), sent as two headers `Revolut-Signature: v1=<hex>` and
    /// `Revolut-Request-Timestamp: <ts_ms>`.
    fn signed_revolut(payload: &[u8], ts: i64, secret: &str) -> WebhookEvent {
        let ts_str = ts.to_string();
        let mut msg = Vec::with_capacity(ts_str.len() + 1 + payload.len());
        msg.extend_from_slice(ts_str.as_bytes());
        msg.push(b'.');
        msg.extend_from_slice(payload);
        let sig = webhook_util::hmac_sha256_hex(secret.as_bytes(), &msg);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("Revolut-Request-Timestamp", ts_str.parse().unwrap());
        headers.insert("Revolut-Signature", format!("v1={sig}").parse().unwrap());
        WebhookEvent {
            provider: "revolut".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    /// Native Revolut events must normalize to the canonical vocabulary the
    /// provider-agnostic dispatch matches on (audit F#11).
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let provider = RevolutProvider::new("k".into(), "whsec".into());
        let now = chrono::Utc::now().timestamp();

        let cases: &[(&str, &str)] = &[
            // Flat Revolut webhook payload, underscore event name (official doc
            // fixture).
            (
                r#"{"event":"ORDER_COMPLETED","order_id":"ord_1","merchant_order_ext_ref":"Test #3928"}"#,
                "checkout.session.completed",
            ),
            // Failure/declined events must NOT map to a grant/payment arm; they
            // fall through to the dispatch's log-only unhandled arm.
            (
                r#"{"event":"ORDER_PAYMENT_FAILED","order_id":"ord_1"}"#,
                "ORDER_PAYMENT_FAILED",
            ),
            (
                r#"{"event":"ORDER_PAYMENT_DECLINED","order_id":"ord_1"}"#,
                "ORDER_PAYMENT_DECLINED",
            ),
            // Unmapped → passthrough.
            (
                r#"{"event":"ORDER_CREATED","order_id":"ord_1"}"#,
                "ORDER_CREATED",
            ),
        ];
        for (body, expected) in cases {
            let evt = signed_revolut(body.as_bytes(), now, "whsec");
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // Structured fields on ORDER_COMPLETED. The flat webhook carries only
        // event + order_id; the grant recovers user_id/amount from the
        // server-bound checkout intent (the dispatch never trusts webhook JSON
        // for granting). The order id round-trips as the checkout intent key.
        let evt = signed_revolut(
            br#"{"event":"ORDER_COMPLETED","order_id":"ord_1","merchant_order_ext_ref":"Test #1"}"#,
            now,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.event_type, "checkout.session.completed");
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("ord_1"));
        assert_eq!(parsed.payment_id.as_deref(), Some("ord_1"));
        // customer_id / user_id / amount / currency are not in the flat payload.
        assert_eq!(parsed.customer_id, "");
        assert_eq!(parsed.user_id, None);
        assert_eq!(parsed.amount_cents, None);

        // Tampered body must be rejected at the signature gate.
        let mut evt = signed_revolut(
            br#"{"event":"ORDER_COMPLETED","order_id":"ord_1"}"#,
            now,
            "whsec",
        );
        evt.payload = br#"{"event":"ORDER_COMPLETED","order_id":"ord_EVIL"}"#.to_vec();
        let err = provider
            .verify_webhook(evt)
            .await
            .expect_err("tampered body rejected");
        assert!(matches!(
            err,
            BillingError::WebhookVerification(msg) if msg.contains("mismatch")
        ));

        // A header without the `v1=` scheme is rejected.
        let mut evt = signed_revolut(
            br#"{"event":"ORDER_COMPLETED","order_id":"ord_1"}"#,
            now,
            "whsec",
        );
        evt.headers.remove("Revolut-Signature");
        evt.headers
            .insert("Revolut-Signature", "deadbeef".parse().unwrap());
        provider
            .verify_webhook(evt)
            .await
            .expect_err("non-v1 scheme rejected");
    }
}
