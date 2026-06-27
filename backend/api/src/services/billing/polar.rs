//! Polar.sh billing provider integration.

use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound Polar call goes through this client (built once in
// `new` with timeouts, or overridden via `with_http_client` with the shared
// AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// Polar.sh billing provider.
///
/// CRYP-ENC-012: `access_token` and `webhook_secret` are held in
/// `secrecy::SecretString` (redacting `Debug`, opt-in `expose_secret()`).
pub struct PolarProvider {
    pub access_token: SecretString,
    pub webhook_secret: SecretString,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl PolarProvider {
    pub fn new(access_token: String, webhook_secret: String) -> Self {
        Self {
            access_token: access_token.into(),
            webhook_secret: webhook_secret.into(),
            // Production by default; override with the sandbox host via
            // POLAR_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("POLAR_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.polar.sh".to_string()),
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

    /// Best-effort fetch of a subscription's `current_period_end` (RFC 3339).
    /// Used on a succeeded `checkout.*` event (whose object is the checkout, not
    /// the subscription, and so carries no period end) to obtain the linked
    /// subscription's authoritative end so the paywall can admit the paying
    /// subscriber. Returns `None` on any failure → fail-closed (audit F#11
    /// round-2).
    async fn fetch_subscription_period_end(&self, subscription_id: &str) -> Option<i64> {
        let client = self.http_client.clone();
        let url = format!("{}/v1/subscriptions/{}", self.base_url, subscription_id);
        let resp = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose_secret()),
            )
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

// CRYP-ENC-012: manual redacting `Debug`. Credential fields are always
// `<redacted>`; only the non-secret wiring is shown for diagnostics. No
// tracing/error path logs the whole struct.
impl std::fmt::Debug for PolarProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolarProvider")
            .field("access_token", &"<redacted>")
            .field("webhook_secret", &"<redacted>")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
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
        let client = self.http_client.clone();
        let body = serde_json::json!({
            "product_id": plan_slug,
            "customer_email": customer_email,
            "metadata": { "user_id": user_id },
            "success_url": success_url,
            "cancel_url": cancel_url,
        });

        let resp = client
            .post(format!("{}/v1/checkouts/", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose_secret()),
            )
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
        let client = self.http_client.clone();
        let url = format!(
            "{}/v1/subscriptions/{}/cancel",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose_secret()),
            )
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
            "{}/v1/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose_secret()),
            )
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
        // Polar.sh follows the Standard Webhooks spec (polar.sh/docs +
        // standardwebhooks.com): the signed message is
        // `"{webhook_id}.{webhook_timestamp}.{body}"`, signed with HMAC-SHA256
        // using the base64-decoded `whsec_<base64>` secret, and the digest is
        // transmitted base64-encoded in the `webhook-signature` header as
        // `v1,<base64>` (possibly multiple entries for key rotation). The three
        // headers `webhook-id`, `webhook-timestamp` (unix seconds) and
        // `webhook-signature` are all required, and the timestamp is bound into
        // the signed message AND checked against a 5-minute replay window.
        //
        // The previous implementation was structurally wrong: it read a
        // non-existent `X-Polar-Signature` header, HMAC'd only the raw body,
        // used the secret's UTF-8 bytes as the key, and compared a hex digest —
        // so every real Polar webhook was rejected (CRYP-HMAC-004).
        let now_secs = chrono::Utc::now().timestamp();
        if !super::webhook_util::verify_standard_webhooks(
            &event.headers,
            self.webhook_secret.expose_secret(),
            &event.payload,
            now_secs,
        ) {
            return Err(BillingError::WebhookVerification(
                "Polar signature verification failed".into(),
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
        let current_period_end =
            match super::provider::period_end_to_unix(obj.get("current_period_end")) {
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
        assert_eq!(provider.access_token.expose_secret(), "access_tok_abc");
        assert_eq!(provider.webhook_secret.expose_secret(), "whsec_xyz");
    }

    use crate::services::billing::webhook_util;
    use base64::Engine;
    use hmac::Mac;

    /// A real Standard Webhooks signer mirroring what Polar.sh actually sends.
    ///
    /// Polar issues the webhook secret as `whsec_<base64>`; the base64 portion
    /// decodes to the 32-byte HMAC key. The signed message is
    /// `"{id}.{ts}.{body}"`, signed with HMAC-SHA256 and transmitted as a
    /// base64 digest in `webhook-signature` as `v1,<base64>` alongside the
    /// `webhook-id` and `webhook-timestamp` headers. This mirrors the spec, NOT
    /// the verifier — so the test exercises a genuine end-to-end check.
    fn sign_polar(payload: &[u8], secret: &str, webhook_id: &str, ts: i64) -> WebhookEvent {
        // Derive the key exactly as the verifier does: strip whsec_, base64-decode.
        let key = webhook_util::standard_webhooks_key(secret);
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&key).unwrap();
        let ts_str = ts.to_string();
        mac.update(webhook_id.as_bytes());
        mac.update(b".");
        mac.update(ts_str.as_bytes());
        mac.update(b".");
        mac.update(payload);
        let sig = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("webhook-id", webhook_id.parse().unwrap());
        headers.insert("webhook-timestamp", ts_str.parse().unwrap());
        headers.insert("webhook-signature", format!("v1,{sig}").parse().unwrap());
        WebhookEvent {
            provider: "polar".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    /// Build a WebhookEvent with arbitrary headers + body (for negative cases).
    fn polar_event(headers: axum::http::HeaderMap, payload: &[u8]) -> WebhookEvent {
        WebhookEvent {
            provider: "polar".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    // ── signature verification (CRYP-HMAC-004) ───────────────────────────────

    #[tokio::test]
    async fn verify_webhook_accepts_valid_signature() {
        // A genuine 32-byte key, base64-encoded and prefixed with whsec_ —
        // exactly how Polar ships the secret.
        let raw_key = [42u8; 32];
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode(raw_key)
        );
        let provider = PolarProvider::new("tok".into(), secret.clone());
        let now = chrono::Utc::now().timestamp();
        let body = br#"{"type":"checkout.updated","data":{"id":"co_1","status":"succeeded","customer_id":"cus_1"}}"#;
        let evt = sign_polar(body, &secret, "evt_abc", now);
        provider
            .verify_webhook(evt)
            .await
            .expect("a correctly signed Standard Webhooks request must verify");
    }

    #[tokio::test]
    async fn verify_webhook_rejects_tampered_body() {
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([7u8; 32])
        );
        let provider = PolarProvider::new("tok".into(), secret.clone());
        let now = chrono::Utc::now().timestamp();
        // Sign one body, then swap in a different one.
        let signed = sign_polar(b"original body", &secret, "evt_1", now);
        let mut evt = signed;
        evt.payload = b"tampered body".to_vec();
        assert!(provider.verify_webhook(evt).await.is_err());
    }

    #[tokio::test]
    async fn verify_webhook_rejects_stale_timestamp() {
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([7u8; 32])
        );
        let provider = PolarProvider::new("tok".into(), secret.clone());
        // 6 minutes in the past — outside the 5-minute replay window.
        let stale = chrono::Utc::now().timestamp() - (webhook_util::MAX_SKEW_SECS + 60);
        let evt = sign_polar(b"{}", &secret, "evt_old", stale);
        assert!(provider.verify_webhook(evt).await.is_err());
    }

    #[tokio::test]
    async fn verify_webhook_rejects_wrong_signature() {
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([7u8; 32])
        );
        let wrong_secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([99u8; 32])
        );
        let provider = PolarProvider::new("tok".into(), wrong_secret);
        let now = chrono::Utc::now().timestamp();
        // Signed with `secret`, verified with `wrong_secret` → mismatch.
        let evt = sign_polar(b"{}", &secret, "evt_1", now);
        assert!(provider.verify_webhook(evt).await.is_err());
    }

    #[tokio::test]
    async fn verify_webhook_rejects_missing_signature_header() {
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([7u8; 32])
        );
        let provider = PolarProvider::new("tok".into(), secret);
        let now = chrono::Utc::now().timestamp();
        // Build the event by hand WITHOUT the webhook-signature header.
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("webhook-id", "evt_1".parse().unwrap());
        headers.insert("webhook-timestamp", now.to_string().parse().unwrap());
        let evt = polar_event(headers, b"{}");
        assert!(provider.verify_webhook(evt).await.is_err());
    }

    #[tokio::test]
    async fn verify_webhook_accepts_rotation_second_key() {
        // Key rotation: the webhook-signature header carries two v1 entries
        // (old + new secret). The verifier must accept if ANY entry matches.
        let old = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([1u8; 32])
        );
        let new = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([2u8; 32])
        );
        let provider = PolarProvider::new("tok".into(), new.clone());
        let now = chrono::Utc::now().timestamp();
        let body = br#"{"type":"checkout.updated","data":{"id":"co_1","status":"succeeded"}}"#;

        // Sign with BOTH keys and concatenate the entries.
        let sig_old = sign_polar(body, &old, "evt_rot", now)
            .headers
            .get("webhook-signature")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let sig_new = sign_polar(body, &new, "evt_rot", now)
            .headers
            .get("webhook-signature")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        // Provider is configured with `new`; the first (old) entry must NOT
        // match, the second (new) entry must. Standard Webhooks SPACE-separates
        // the rotation entries (the comma is inside each `v1,<sig>`), so the
        // combined header must mirror the real wire format.
        let combined = format!("{sig_old} {sig_new}");

        let mut headers = axum::http::HeaderMap::new();
        headers.insert("webhook-id", "evt_rot".parse().unwrap());
        headers.insert("webhook-timestamp", now.to_string().parse().unwrap());
        headers.insert("webhook-signature", combined.parse().unwrap());
        let evt = polar_event(headers, body);
        provider
            .verify_webhook(evt)
            .await
            .expect("must accept when the rotation entry (second v1) matches");
    }

    /// Native Polar events must normalize to the canonical vocabulary the
    /// provider-agnostic dispatch matches on (audit F#11). Critically,
    /// `checkout.updated` fires on ANY status transition, so it must only map to
    /// CHECKOUT_COMPLETED when `status == "succeeded"` — otherwise a pre-payment
    /// `open` update would consume the single-use intent and grant prematurely.
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode([7u8; 32])
        );
        let provider = PolarProvider::new("tok".into(), secret.clone());
        let now = chrono::Utc::now().timestamp();

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
            let evt = sign_polar(body.as_bytes(), &secret, "evt_norm", now);
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // GATING: a checkout.updated that has NOT yet succeeded must NOT map to
        // CHECKOUT_COMPLETED (would prematurely consume the single-use intent).
        let evt = sign_polar(
            br#"{"type":"checkout.updated","data":{"id":"co_1","status":"open"}}"#,
            &secret,
            "evt_gate",
            now,
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.event_type, "checkout.updated");

        // Structured fields on a succeeded checkout.
        let evt = sign_polar(
            br#"{"type":"checkout.updated","data":{"id":"co_1","status":"succeeded","customer_id":"cus_1","subscription_id":"sub_1","metadata":{"user_id":"42"},"total_amount":9999,"currency":"usd"}}"#,
            &secret,
            "evt_fields",
            now,
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
