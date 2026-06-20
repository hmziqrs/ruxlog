//! Airwallex billing provider integration (Global/APAC).
//!
//! Supports multi-currency, cross-border payments, and subscriptions.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound Airwallex call goes through this client (built once
// in `new` with timeouts, or overridden via `with_http_client` with the shared
// AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// Airwallex billing provider.
pub struct AirwallexProvider {
    pub client_id: String,
    pub api_key: String,
    pub webhook_secret: String,
    pub base_url: String,
    /// Hosted-checkout / customer-portal base URL. Production by default
    /// (`https://checkout.airwallex.com`); override with the sandbox host
    /// (`https://demo.airwallex.com`) via AIRWALLEX_CHECKOUT_BASE_URL for
    /// development. See plan Phase 6f — this was previously hardcoded to the
    /// demo host, which sent production shoppers to the sandbox checkout.
    pub checkout_base_url: String,
    pub http_client: reqwest::Client,
}

impl AirwallexProvider {
    pub fn new(client_id: String, api_key: String, webhook_secret: String) -> Self {
        Self {
            client_id,
            api_key,
            webhook_secret,
            // Production by default; override with the sandbox URL via
            // AIRWALLEX_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("AIRWALLEX_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.airwallex.com/api/v1".to_string()),
            // Hosted-checkout base URL, env-driven with a production default.
            // (Phase 6f: previously hardcoded to the demo/sandbox host.)
            checkout_base_url: std::env::var("AIRWALLEX_CHECKOUT_BASE_URL")
                .unwrap_or_else(|_| "https://checkout.airwallex.com".to_string()),
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

impl AirwallexProvider {
    /// Authenticate and get a bearer token from Airwallex.
    async fn get_access_token(&self) -> Result<String, BillingError> {
        let client = self.http_client.clone();
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
        let client = self.http_client.clone();

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

        // Generate the hosted-checkout URL from the env-driven base (production
        // by default); AIRWALLEX_CHECKOUT_BASE_URL overrides for the sandbox.
        let checkout_url = format!(
            "{}/checkout?intent_id={}&client_secret={}",
            self.checkout_base_url,
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
        let client = self.http_client.clone();
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
        let client = self.http_client.clone();
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
        // Airwallex sends TWO separate headers (official Airwallex webhook docs,
        // "Verify the signature"): `x-timestamp` (epoch-millis string) and
        // `x-signature` (the hex HMAC). The value to digest is the concatenation
        // of the timestamp string and the raw request body:
        //   value_to_digest = x-timestamp + request_body
        //   x-signature     = HMAC-SHA256(webhook_secret, value_to_digest)
        // The previous code read a single non-existent `x-www-airwallex-signature`
        // header and expected a `<ts>.<hmac>` shape Airwallex never sends — so
        // every webhook died at this gate (audit F#11 round-2). Only the header
        // *names* were wrong; the digest construction below was already correct.
        let ts = super::webhook_util::header_str(&event.headers, "x-timestamp")
            .ok_or_else(|| BillingError::WebhookVerification("Missing x-timestamp header".into()))?;
        let mac_hex = super::webhook_util::header_str(&event.headers, "x-signature")
            .ok_or_else(|| BillingError::WebhookVerification("Missing x-signature header".into()))?;

        // Replay protection. Airwallex sends epoch milliseconds; normalize to
        // seconds via the same magnitude heuristic as `period_end_to_unix`.
        let ts_raw: i64 = ts.trim().parse().map_err(|_| {
            BillingError::WebhookVerification("Airwallex timestamp not an integer".into())
        })?;
        let ts_secs = if ts_raw > 1_000_000_000_000 { ts_raw / 1000 } else { ts_raw };
        if !super::webhook_util::timestamp_fresh(ts_secs, chrono::Utc::now().timestamp()) {
            return Err(BillingError::WebhookVerification(format!(
                "Airwallex timestamp outside tolerance (ts={ts_secs})"
            )));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        // value_to_digest = x-timestamp string + raw body (no separator).
        let mut manifest = Vec::with_capacity(ts.len() + event.payload.len());
        manifest.extend_from_slice(ts.as_bytes());
        manifest.extend_from_slice(&event.payload);
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.as_bytes(),
            &manifest,
            &mac_hex,
        ) {
            return Err(BillingError::WebhookVerification(
                "Airwallex signature mismatch".into(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Normalize Airwallex's native event taxonomy to the canonical vocabulary
        // the dispatch matches on (audit F#11 residual). Airwallex checkouts are
        // payment intents; `payment_intent.succeeded` is the checkout-completion
        // signal and carries the payment intent id we keyed the intent by.
        let native_event = data["event_type"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "payment_intent.succeeded" => super::provider::canonical::CHECKOUT_COMPLETED,
            "subscription.succeeded" | "subscription.updated" => {
                super::provider::canonical::SUBSCRIPTION_UPDATED
            }
            // Cancellation/expiry lifecycle events (audit F#11 residual: these
            // were missing, so a cancelled/expired Airwallex subscription never
            // reached the SUBSCRIPTION_DELETED arm and the row stayed active).
            // Accept both British and American spellings.
            "subscription.cancelled" | "subscription.canceled" | "subscription.expired" => {
                super::provider::canonical::SUBSCRIPTION_DELETED
            }
            other => other,
        }
        .to_string();

        let obj = &data["data"]["entity"];
        let intent_id = obj["payment_intent_id"]
            .as_str()
            .or_else(|| obj["id"].as_str())
            .map(String::from);
        // On `subscription.*` lifecycle events the entity IS the subscription,
        // so its `id` is the provider subscription id (the explicit
        // `subscription_id` field is absent then). On `payment_intent.*` events
        // the entity is the intent — its `id` is NOT a subscription, so the
        // fallback must be gated to lifecycle events only. Hoisted out of the
        // struct literal so it doesn't borrow `event_type` after its move.
        let lifecycle_subscription_id = if native_event.starts_with("subscription.") {
            obj["id"].as_str().map(String::from)
        } else {
            None
        };

        Ok(ParsedWebhook {
            event_type,
            customer_id: obj["customer_id"].as_str().unwrap_or_default().to_string(),
            // `subscription_id` is present when the event references a
            // subscription; on lifecycle events fall back to the entity `id`
            // (audit F#11 round-2: the row's `subscription_id` was `None` for
            // lifecycle events, so the SUBSCRIPTION_UPDATED/DELETED dispatch arm
            // no-oped and cancelled subs stayed active).
            subscription_id: obj["subscription_id"]
                .as_str()
                .map(String::from)
                .or(lifecycle_subscription_id),
            payment_id: intent_id.clone(),
            // Airwallex keys the checkout intent by the payment intent id (its
            // `create_checkout` returns it as `session_id`); the entity echoes
            // it back as `payment_intent_id`.
            checkout_session_id: intent_id,
            // Airwallex subscription events carry `current_period_end` (RFC 3339)
            // on the entity when present.
            current_period_end: super::provider::period_end_to_unix(obj.get("current_period_end")),
            subscription_status: obj["subscription_status"]
                .as_str()
                .or_else(|| obj["status"].as_str())
                .map(String::from),
            // Airwallex's `merchant_order_no` is set at create-checkout as
            // `order_{user_id}_{timestamp}` (see `create_checkout`); recover the
            // user_id segment. A bare numeric id also parses.
            user_id: obj["merchant_order_no"].as_str().and_then(|s| {
                s.strip_prefix("order_")
                    .and_then(|rest| rest.split('_').next())
                    .unwrap_or(s)
                    .parse()
                    .ok()
            }),
            amount_cents: obj["amount"].as_i64(),
            currency: obj["currency"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // Airwallex doesn't provide a native billing portal; build the
        // customer URL from the env-driven checkout base (production default).
        Ok(format!(
            "{}/customer/{}?return_url={}",
            self.checkout_base_url,
            provider_customer_id,
            urlencoding::encode(return_url)
        ))
    }
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
    fn checkout_base_url_defaults_to_production_not_sandbox() {
        // Phase 6f: the hosted-checkout host must default to the production
        // checkout.airwallex.com, NOT the demo/sandbox host that was hardcoded
        // before. `new()` reads the env at construction time.
        std::env::remove_var("AIRWALLEX_CHECKOUT_BASE_URL");
        let provider = AirwallexProvider::new("c".into(), "k".into(), "w".into());
        assert_eq!(provider.checkout_base_url, "https://checkout.airwallex.com");
        assert!(
            !provider.checkout_base_url.contains("demo"),
            "must not default to the sandbox host: {}",
            provider.checkout_base_url
        );
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

    use crate::services::billing::webhook_util;

    /// Sign an Airwallex webhook exactly as Airwallex does: two separate
    /// headers `x-timestamp` + `x-signature`, where the signature is
    /// HMAC-SHA256(webhook_secret, value_to_digest) and
    /// value_to_digest = x-timestamp string + raw body.
    fn signed_airwallex(payload: &[u8], ts: i64, secret: &str) -> WebhookEvent {
        let ts_str = ts.to_string();
        let mut msg = Vec::with_capacity(ts_str.len() + payload.len());
        msg.extend_from_slice(ts_str.as_bytes());
        msg.extend_from_slice(payload);
        let sig = webhook_util::hmac_sha256_hex(secret.as_bytes(), &msg);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-timestamp", ts_str.parse().unwrap());
        headers.insert("x-signature", sig.parse().unwrap());
        WebhookEvent {
            provider: "airwallex".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    /// Native Airwallex events must normalize to the canonical vocabulary the
    /// provider-agnostic dispatch matches on (audit F#11).
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let provider = AirwallexProvider::new("c".into(), "k".into(), "whsec".into());
        let now = chrono::Utc::now().timestamp();

        let cases: &[(&str, &str)] = &[
            (
                r#"{"event_type":"payment_intent.succeeded","data":{"entity":{"payment_intent_id":"int_1","status":"SUCCEEDED","merchant_order_no":"order_42","amount":9999,"currency":"USD"}}}"#,
                "checkout.session.completed",
            ),
            (
                r#"{"event_type":"subscription.succeeded","data":{"entity":{"subscription_id":"sub_2","status":"ACTIVE"}}}"#,
                "customer.subscription.updated",
            ),
            (
                r#"{"event_type":"subscription.updated","data":{"entity":{"subscription_id":"sub_2","status":"ACTIVE"}}}"#,
                "customer.subscription.updated",
            ),
            // Cancellation/expiry reach the deleted arm (audit F#11 lifecycle).
            (
                r#"{"event_type":"subscription.cancelled","data":{"entity":{"subscription_id":"sub_3","status":"CANCELLED"}}}"#,
                "customer.subscription.deleted",
            ),
            (
                r#"{"event_type":"subscription.expired","data":{"entity":{"subscription_id":"sub_3","status":"EXPIRED"}}}"#,
                "customer.subscription.deleted",
            ),
            // Unmapped → passthrough.
            (
                r#"{"event_type":"refund.created","data":{"entity":{}}}"#,
                "refund.created",
            ),
        ];
        for (body, expected) in cases {
            let evt = signed_airwallex(body.as_bytes(), now, "whsec");
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // Structured fields on a checkout-completion (payment intent) event.
        let evt = signed_airwallex(
            br#"{"event_type":"payment_intent.succeeded","data":{"entity":{"payment_intent_id":"int_1","status":"SUCCEEDED","merchant_order_no":"order_42_1700000000","amount":9999,"currency":"USD"}}}"#,
            now,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("int_1"));
        assert_eq!(parsed.payment_id.as_deref(), Some("int_1"));
        assert_eq!(parsed.subscription_status.as_deref(), Some("SUCCEEDED"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(9999));
        assert_eq!(parsed.currency.as_deref(), Some("USD"));

        // Realistic Airwallex payment-intent entities carry `id` (not a
        // redundant `payment_intent_id`); the fallback round-trips it as the
        // checkout intent key (audit F#11 round-2).
        let evt = signed_airwallex(
            br#"{"event_type":"payment_intent.succeeded","data":{"entity":{"id":"int_real","status":"SUCCEEDED","merchant_order_no":"order_7_1700000000"}}}"#,
            now,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("int_real"));
        assert_eq!(parsed.payment_id.as_deref(), Some("int_real"));

        // Lifecycle event whose entity IS the subscription (no explicit
        // `subscription_id`): the `id` fallback populates `subscription_id` so
        // the SUBSCRIPTION_DELETED dispatch arm can act (audit F#11 round-2).
        let evt = signed_airwallex(
            br#"{"event_type":"subscription.cancelled","data":{"entity":{"id":"sub_xyz","status":"CANCELLED"}}}"#,
            now,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.event_type, "customer.subscription.deleted");
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_xyz"));

        // A payment_intent event must NOT pick up its entity `id` as a
        // subscription id (the fallback is gated to lifecycle events only).
        let evt = signed_airwallex(
            br#"{"event_type":"payment_intent.succeeded","data":{"entity":{"id":"int_no_sub","status":"SUCCEEDED"}}}"#,
            now,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.subscription_id, None);
    }
}
