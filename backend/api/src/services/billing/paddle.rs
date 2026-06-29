//! Paddle billing provider integration.

use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound Paddle call goes through this client (built once in
// `new` with timeouts, or overridden via `with_http_client` with the shared
// AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// Paddle billing provider.
///
/// CRYP-ENC-012: `client_token` and `webhook_secret` are held in
/// `secrecy::SecretString` (redacting `Debug`, opt-in `expose_secret()`).
pub struct PaddleProvider {
    pub client_token: SecretString,
    pub webhook_secret: SecretString,
    /// Ed25519 verifying key (32 bytes) for Paddle webhook signatures, parsed
    /// from `PADDLE_PUBLIC_KEY` (hex). `None` ⇒ verification fails closed.
    pub public_key: Option<[u8; 32]>,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl PaddleProvider {
    pub fn new(client_token: String, webhook_secret: String) -> Self {
        Self {
            client_token: client_token.into(),
            webhook_secret: webhook_secret.into(),
            public_key: None,
            // Production by default; override with the sandbox host via
            // PADDLE_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("PADDLE_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.paddle.com".to_string()),
            http_client: build_http_client(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Set the Ed25519 public key used to verify Paddle webhooks (hex of 32 bytes).
    pub fn with_public_key(mut self, hex_key: &str) -> Result<Self, BillingError> {
        self.public_key = Some(decode_paddle_public_key(hex_key)?);
        Ok(self)
    }

    /// V-MED-10: inject the shared, timeout-configured client from `AppState`.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    pub fn from_env() -> Result<Self, BillingError> {
        let client_token = std::env::var("PADDLE_CLIENT_TOKEN")
            .map_err(|_| BillingError::Config("PADDLE_CLIENT_TOKEN not set".into()))?;
        let webhook_secret = std::env::var("PADDLE_WEBHOOK_SECRET")
            .map_err(|_| BillingError::Config("PADDLE_WEBHOOK_SECRET not set".into()))?;
        let mut provider = Self::new(client_token, webhook_secret);
        match std::env::var("PADDLE_PUBLIC_KEY") {
            Ok(hex_key) => provider = provider.with_public_key(&hex_key)?,
            Err(_) => tracing::warn!(
                "PADDLE_PUBLIC_KEY not set; Paddle webhooks will fail verification until it is configured"
            ),
        }
        Ok(provider)
    }

    /// Best-effort fetch of a subscription's current period end. Used on
    /// `transaction.*` checkout-completion events, which are transactions, not
    /// subscriptions, and so don't carry a billing period — without it the
    /// paywall would deny the paying subscriber (it fails closed on a missing
    /// period end). Returns `None` on any failure (network, non-2xx, missing
    /// field) so the grant degrades to fail-closed rather than synthesizing a
    /// period end.
    async fn fetch_subscription_period_end(
        &self,
        subscription_id: &str,
    ) -> Option<serde_json::Value> {
        let client = self.http_client.clone();
        let url = format!("{}/subscriptions/{}", self.base_url, subscription_id);
        let resp = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client_token.expose_secret()),
            )
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let data: serde_json::Value = resp.json().await.ok()?;
        data.get("data")
            .and_then(|d| d.get("current_billing_period"))
            .and_then(|c| c.get("ends_at"))
            .cloned()
    }
}

/// Decode the Paddle Ed25519 public key from hex (64 chars → 32 bytes).
fn decode_paddle_public_key(hex_key: &str) -> Result<[u8; 32], BillingError> {
    let bytes = hex::decode(hex_key.trim())
        .map_err(|_| BillingError::Config("PADDLE_PUBLIC_KEY is not valid hex".into()))?;
    if bytes.len() != 32 {
        return Err(BillingError::Config(format!(
            "PADDLE_PUBLIC_KEY must be 32 bytes (64 hex chars), got {} bytes",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

// CRYP-ENC-012: manual redacting `Debug`. Credential fields are always
// `<redacted>`; only the non-secret wiring is shown. No tracing/error path
// logs the whole struct.
impl std::fmt::Debug for PaddleProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaddleProvider")
            .field("client_token", &"<redacted>")
            .field("webhook_secret", &"<redacted>")
            .field("public_key_set", &self.public_key.is_some())
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl BillingProvider for PaddleProvider {
    fn provider_name(&self) -> &'static str {
        "paddle"
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
            "items": [{ "price_id": plan_slug, "quantity": 1 }],
            "custom_data": { "user_id": user_id.to_string() },
            "customer_email": customer_email,
            "urls": {
                "success_url": success_url,
                "cancel_url": cancel_url,
            }
        });

        let resp = client
            .post(format!("{}/transactions", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.client_token.expose_secret()),
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
            session_id: data["data"]["id"].as_str().unwrap_or_default().to_string(),
            checkout_url: data["data"]["checkout"]["url"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let client = self.http_client.clone();
        let url = format!(
            "{}/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let body = if immediately {
            serde_json::json!({ "status": "canceled" })
        } else {
            serde_json::json!({ "scheduled_change": { "action": "cancel" } })
        };

        let resp = client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client_token.expose_secret()),
            )
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
        let client = self.http_client.clone();
        let url = format!(
            "{}/subscriptions/{}",
            self.base_url, provider_subscription_id
        );

        let resp = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client_token.expose_secret()),
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

        let d = &data["data"];
        Ok(SubscriptionInfo {
            provider_subscription_id: d["id"].as_str().unwrap_or_default().to_string(),
            status: d["status"].as_str().unwrap_or_default().to_string(),
            current_period_end: d["next_billed_at"].as_str().and_then(|s| s.parse().ok()),
            cancel_at_period_end: d["scheduled_change"]["action"]
                .as_str()
                .map(|a| a == "cancel")
                .unwrap_or(false),
        })
    }

    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError> {
        // Paddle Billing signs "<timestamp><raw_body>" with Ed25519 and sends
        // `Paddle-Signature: ts=<ts>;key1=<hexsig>`. Verification is mandatory:
        // the previous code skipped it entirely when the header was empty — a
        // fail-open that accepted any forged JSON.
        let public_key = self.public_key.ok_or_else(|| {
            BillingError::WebhookVerification(
                "PADDLE_PUBLIC_KEY not configured; cannot verify Paddle webhook".into(),
            )
        })?;

        let sig_header = super::webhook_util::header_str(&event.headers, "Paddle-Signature")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing Paddle-Signature header".into())
            })?;

        // Parse "ts=<ts>;key1=<hexsig>".
        let mut ts: Option<&str> = None;
        let mut key1: Option<&str> = None;
        for part in sig_header.split(';') {
            if let Some((k, v)) = part.split_once('=') {
                match k.trim() {
                    "ts" => ts = Some(v.trim()),
                    "key1" => key1 = Some(v.trim()),
                    _ => {}
                }
            }
        }
        let ts = ts.ok_or_else(|| {
            BillingError::WebhookVerification("Paddle-Signature missing ts=".into())
        })?;
        let sig_hex = key1.ok_or_else(|| {
            BillingError::WebhookVerification("Paddle-Signature missing key1=".into())
        })?;

        // Replay protection.
        let ts_secs: i64 = ts.parse().map_err(|_| {
            BillingError::WebhookVerification("Paddle timestamp not an integer".into())
        })?;
        if !super::webhook_util::timestamp_fresh(ts_secs, chrono::Utc::now().timestamp()) {
            return Err(BillingError::WebhookVerification(format!(
                "Paddle timestamp outside tolerance (ts={ts_secs})"
            )));
        }

        // Decode the 64-byte Ed25519 signature.
        let sig_bytes = hex::decode(sig_hex).map_err(|_| {
            BillingError::WebhookVerification("Paddle signature not valid hex".into())
        })?;
        let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().map_err(|_| {
            BillingError::WebhookVerification("Paddle signature must be 64 bytes".into())
        })?;

        // Signed message = "<ts><raw_body>".
        let mut message = Vec::with_capacity(ts.len() + event.payload.len());
        message.extend_from_slice(ts.as_bytes());
        message.extend_from_slice(&event.payload);
        if !super::webhook_util::verify_ed25519(&public_key, &message, &sig_arr) {
            return Err(BillingError::WebhookVerification(
                "Paddle signature mismatch".into(),
            ));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        let data: serde_json::Value = serde_json::from_str(payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let obj = &data["data"];

        // Normalize Paddle's native event taxonomy to the canonical vocabulary
        // the dispatch matches on (audit F#11 residual). Paddle fires
        // `transaction.completed` when a checkout is paid and fully processed —
        // that is the terminal grant signal and carries the transaction id we
        // stored the intent under. Only `transaction.completed` (NOT
        // `transaction.paid`, which can fire before the linked subscription is
        // populated and would grant without a resolvable period end) maps to
        // CHECKOUT_COMPLETED (audit F#11 round-2). `subscription.*` events are
        // lifecycle.
        let native_event = data["event_type"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "transaction.completed" => super::provider::canonical::CHECKOUT_COMPLETED,
            "subscription.updated" => super::provider::canonical::SUBSCRIPTION_UPDATED,
            "subscription.canceled" => super::provider::canonical::SUBSCRIPTION_DELETED,
            other => other,
        }
        .to_string();

        // Resolve the current period end. Subscription events carry it inline as
        // `current_billing_period.ends_at` (RFC 3339). A `transaction.completed`
        // checkout event is a transaction, not a subscription, and carries no
        // period — so fetch the linked subscription (`subscription_id`) for the
        // authoritative end. Without it the paywall would deny the paying
        // subscriber (it fails closed on a missing period end). Fetch failures
        // degrade to None (fail-closed). The fetched value is bound to a local so
        // it outlives the borrow passed to `period_end_to_unix`.
        let inline_end = obj
            .get("current_billing_period")
            .and_then(|c| c.get("ends_at"));
        let fetched_end: Option<serde_json::Value> = match inline_end {
            Some(_) => None,
            None => match obj.get("subscription_id").and_then(|s| s.as_str()) {
                Some(sub_id) => self.fetch_subscription_period_end(sub_id).await,
                None => None,
            },
        };
        let period_end_value = inline_end.or(fetched_end.as_ref());

        // `subscription_id`: on `transaction.*` events the object IS a
        // transaction, so its `id` is the txn id (the checkout intent key), NOT a
        // subscription — the linked subscription is `subscription_id`. On
        // `subscription.*` lifecycle events the object IS the subscription, so
        // its `id` is the provider subscription id (the explicit field is absent
        // then). Gate the `id` fallback to lifecycle events so a transaction
        // never reports its txn id as a subscription id (audit F#11 round-2: the
        // SUBSCRIPTION_UPDATED/DELETED dispatch arm no-oped for lifecycle events
        // whose `subscription_id` field was absent).
        let lifecycle_subscription_id = if native_event.starts_with("subscription.") {
            obj["id"].as_str().map(String::from)
        } else {
            None
        };

        Ok(ParsedWebhook {
            event_type,
            customer_id: obj["customer_id"].as_str().unwrap_or_default().to_string(),
            subscription_id: obj["subscription_id"]
                .as_str()
                .map(String::from)
                .or(lifecycle_subscription_id),
            payment_id: obj["transaction_id"]
                .as_str()
                .map(String::from)
                .or_else(|| obj["id"].as_str().map(String::from)),
            current_period_end: super::provider::period_end_to_unix(period_end_value),
            // On a `transaction.*` event the object id IS the transaction id we
            // keyed the intent by at create-checkout (`data.id`).
            checkout_session_id: obj["id"].as_str().map(String::from),
            subscription_status: obj["status"].as_str().map(String::from),
            user_id: obj["custom_data"]["user_id"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            amount_cents: obj["details"]["totals"]["total"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .or_else(|| obj["total"].as_i64()),
            currency: obj["currency_code"].as_str().map(String::from),
            data,
        })
    }

    async fn create_portal_session(
        &self,
        _provider_customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        Err(BillingError::InvalidRequest(
            "Paddle uses its own customer portal".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::billing::webhook_util;

    #[test]
    fn test_paddle_provider_name() {
        let provider = PaddleProvider::new("paddle_token".into(), "paddle_secret".into());
        assert_eq!(provider.provider_name(), "paddle");
    }

    #[test]
    fn test_paddle_new() {
        let provider = PaddleProvider::new("tok_abc".into(), "whsec_xyz".into());
        assert_eq!(provider.client_token.expose_secret(), "tok_abc");
        assert_eq!(provider.webhook_secret.expose_secret(), "whsec_xyz");
    }

    #[test]
    fn test_paddle_from_env_missing() {
        std::env::remove_var("PADDLE_CLIENT_TOKEN");
        std::env::remove_var("PADDLE_WEBHOOK_SECRET");
        let result = PaddleProvider::from_env();
        assert!(result.is_err());
    }

    /// Sign a Paddle webhook exactly as Paddle does: Ed25519 over `<ts><body>`,
    /// header `Paddle-Signature: ts=<ts>;key1=<hexsig>`.
    fn signed_paddle_event(
        payload: &[u8],
        ts: i64,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> WebhookEvent {
        use ed25519_dalek::Signer;
        let ts_str = ts.to_string();
        let mut msg = Vec::with_capacity(ts_str.len() + payload.len());
        msg.extend_from_slice(ts_str.as_bytes());
        msg.extend_from_slice(payload);
        let sig = signing_key.sign(&msg);

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "Paddle-Signature",
            format!("ts={ts_str};key1={}", hex::encode(sig.to_bytes()))
                .parse()
                .unwrap(),
        );
        WebhookEvent {
            provider: "paddle".into(),
            payload: payload.to_vec(),
            headers,
            query: None,
        }
    }

    #[tokio::test]
    async fn verify_webhook_accepts_genuine_and_rejects_tampered() {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let vk = sk.verifying_key();
        let provider = PaddleProvider::new("tok".into(), "whsec".into())
            .with_public_key(&hex::encode(vk.to_bytes()))
            .expect("valid hex public key");

        let body = br#"{"event_type":"subscription.created","data":{"id":"sub_1","customer_id":"ctm_1","transaction_id":"txn_1"}}"#;
        let now = chrono::Utc::now().timestamp();
        let genuine = signed_paddle_event(body, now, &sk);

        // Genuine Ed25519 signature → accepted.
        let parsed = provider
            .verify_webhook(genuine.clone())
            .await
            .expect("genuine Paddle webhook must verify");
        assert_eq!(parsed.event_type, "subscription.created");
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_1"));

        // Tampered body under the ORIGINAL signature → rejected.
        let mut tampered = genuine.clone();
        tampered.payload =
            br#"{"event_type":"subscription.created","data":{"id":"sub_EVIL"}}"#.to_vec();
        assert!(provider.verify_webhook(tampered).await.is_err());

        // Stale timestamp → rejected (replay).
        let stale = signed_paddle_event(body, now - (webhook_util::MAX_SKEW_SECS + 60), &sk);
        assert!(provider.verify_webhook(stale).await.is_err());
    }

    /// Native Paddle events must normalize to the canonical vocabulary the
    /// provider-agnostic dispatch matches on (audit F#11).
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key();
        let provider = PaddleProvider::new("tok".into(), "whsec".into())
            .with_public_key(&hex::encode(vk.to_bytes()))
            .expect("valid hex public key");
        let now = chrono::Utc::now().timestamp();

        let cases: &[(&str, &str)] = &[
            // A paid checkout: the terminal transaction-completion event is the
            // grant signal and carries the txn id we keyed the intent by.
            (
                r#"{"event_type":"transaction.completed","data":{"id":"txn_1","customer_id":"ctm_1","subscription_id":"sub_9","status":"completed","currency_code":"USD","details":{"totals":{"total":"1000"}}}}"#,
                "checkout.session.completed",
            ),
            // `transaction.paid` is NOT a grant (it can fire before the linked
            // subscription is populated → grant without a period end). It passes
            // through to the dispatch's log-only unhandled arm (audit F#11
            // round-2); the grant happens on `transaction.completed`.
            (
                r#"{"event_type":"transaction.paid","data":{"id":"txn_2"}}"#,
                "transaction.paid",
            ),
            (
                r#"{"event_type":"subscription.updated","data":{"id":"sub_2","status":"active"}}"#,
                "customer.subscription.updated",
            ),
            (
                r#"{"event_type":"subscription.canceled","data":{"id":"sub_2","status":"canceled"}}"#,
                "customer.subscription.deleted",
            ),
            // An unmapped event passes through (logged as unhandled, not dropped
            // into a canonical arm).
            (
                r#"{"event_type":"reporting.transaction.created","data":{"id":"x"}}"#,
                "reporting.transaction.created",
            ),
        ];
        for (body, expected) in cases {
            let evt = signed_paddle_event(body.as_bytes(), now, &sk);
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // Structured fields on a checkout-completion event. `subscription_id`
        // is resolved from `data.subscription_id` (the linked sub), NOT
        // `data.id` (which is the txn id / intent key).
        let evt = signed_paddle_event(
            br#"{"event_type":"transaction.completed","data":{"id":"txn_1","subscription_id":"sub_9","custom_data":{"user_id":"42"},"status":"completed","currency_code":"USD","details":{"totals":{"total":"1000"}}}}"#,
            now,
            &sk,
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("txn_1"));
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_9"));
        assert_eq!(parsed.subscription_status.as_deref(), Some("completed"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(1000));
        assert_eq!(parsed.currency.as_deref(), Some("USD"));

        // Lifecycle event whose object IS the subscription: the `id` fallback
        // populates `subscription_id` (audit F#11 round-2).
        let evt = signed_paddle_event(
            br#"{"event_type":"subscription.canceled","data":{"id":"sub_zz","status":"canceled"}}"#,
            now,
            &sk,
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.event_type, "customer.subscription.deleted");
        assert_eq!(parsed.subscription_id.as_deref(), Some("sub_zz"));
    }

    #[tokio::test]
    async fn verify_webhook_rejects_when_public_key_unconfigured() {
        // No PADDLE_PUBLIC_KEY → fail closed even with a well-formed header.
        let provider = PaddleProvider::new("tok".into(), "whsec".into());
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("Paddle-Signature", "ts=1;key1=00".parse().unwrap());
        let event = WebhookEvent {
            provider: "paddle".into(),
            payload: b"{}".to_vec(),
            headers,
            query: None,
        };
        assert!(provider.verify_webhook(event).await.is_err());
    }
}
