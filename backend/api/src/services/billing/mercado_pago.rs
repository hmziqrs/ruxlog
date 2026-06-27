//! Mercado Pago billing provider integration (LATAM).
//!
//! Supports PIX, OXXO, credit cards, and Checkout Pro.

use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound Mercado Pago call goes through this client (built
// once in `new` with timeouts, or overridden via `with_http_client` with the
// shared AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// Mercado Pago billing provider.
///
/// CRYP-ENC-012: `access_token` and `webhook_secret` are held in
/// `secrecy::SecretString` (redacting `Debug`, opt-in `expose_secret()`).
pub struct MercadoPagoProvider {
    pub access_token: SecretString,
    pub webhook_secret: SecretString,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

/// Extract a parameter from a raw URL query string, returning the first match.
/// Handles `+`-encoded spaces and percent-decoding (V-CRIT-2). Returns the raw
/// (decoded) value without lowercasing — the caller decides casing.
fn extract_query_param(query: &str, name: &str) -> Option<String> {
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next()? == name {
            let val = kv.next()?;
            return Some(url_decode(val));
        }
    }
    None
}

/// Minimal percent-decode + `+`-to-space (form-encoded) using only std, so we
/// don't pull in a new direct dependency for a single webhook param. (V-CRIT-2)
fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                if let Some(b) = hex_nibble(bytes[i + 1])
                    .and_then(|hi| hex_nibble(bytes[i + 2]).map(|lo| (hi << 4) | lo))
                {
                    out.push(b);
                    i += 2;
                } else {
                    // Malformed escape — preserve literally (lenient).
                    out.push(b'%');
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Normalise the ambiguous `ts=` value from `x-signature` to whole seconds
/// (V-CRIT-2 residual). Mercado Pago's documented example uses a 10-digit
/// SECONDS value (ts=1704908010); some integrations emit a 13-digit
/// millisecond value. Any value larger than 10_000_000_000 (≈ year 2286 in
/// seconds) is assumed to be milliseconds and divided by 1000; otherwise it is
/// treated as seconds verbatim. Picking 10_000_000_000 as the threshold keeps
/// genuine seconds values (which will not exceed it for centuries) intact
/// while correctly collapsing millisecond timestamps.
fn ts_unit_to_secs(ts: i64) -> i64 {
    if ts > 10_000_000_000 {
        ts / 1000
    } else {
        ts
    }
}

impl MercadoPagoProvider {
    pub fn new(access_token: String, webhook_secret: String) -> Self {
        Self {
            access_token: access_token.into(),
            webhook_secret: webhook_secret.into(),
            // Production by default; override with the sandbox host via
            // MERCADO_PAGO_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("MERCADO_PAGO_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.mercadopago.com".to_string()),
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

// CRYP-ENC-012: manual redacting `Debug`. Credential fields are always
// `<redacted>`; only the non-secret wiring is shown. No tracing/error path
// logs the whole struct.
impl std::fmt::Debug for MercadoPagoProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MercadoPagoProvider")
            .field("access_token", &"<redacted>")
            .field("webhook_secret", &"<redacted>")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
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
        let client = self.http_client.clone();

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
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose_secret()),
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
            checkout_url: data["init_point"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        _immediately: bool,
    ) -> Result<(), BillingError> {
        let client = self.http_client.clone();
        let url = format!("{}/preapproval/{}", self.base_url, provider_subscription_id);

        let resp = client
            .put(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose_secret()),
            )
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
        let client = self.http_client.clone();
        let url = format!("{}/preapproval/{}", self.base_url, provider_subscription_id);

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
        // Production path: pin "now" to the wall clock. Tests call
        // `verify_webhook_at` with a fixed `now_secs` so they are deterministic
        // (V-CRIT-2 residuals: ts-unit disambiguation + data.id verbatim).
        self.verify_webhook_at(event, chrono::Utc::now().timestamp())
            .await
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

/// Inherent helpers for [`MercadoPagoProvider`].
///
/// `verify_webhook_at` is split out of the `BillingProvider` impl so tests can
/// pin `now_secs` (production pins it to the wall clock via the trait's
/// `verify_webhook` above). It is an inherent method because `BillingProvider`
/// only declares `verify_webhook` — defining it inside the trait impl block is
/// not valid Rust.
impl MercadoPagoProvider {
    pub async fn verify_webhook_at(
        &self,
        event: WebhookEvent,
        now_secs: i64,
    ) -> Result<ParsedWebhook, BillingError> {
        // Mercado Pago signs with x-signature: "ts=<ts>,v1=<hmac-sha256>". The
        // tag is HMAC-SHA256(secret, "id:{data.id};request-id:{x-request-id};ts:{ts};")
        // (V-CRIT-2), built from the URL query data.id + x-request-id header +
        // the ts= parsed below.
        //
        // ts UNIT (V-CRIT-2 residual): Mercado Pago's own documentation example
        // uses a 10-digit SECONDS value (ts=1704908010 ≈ 2024-01-10). Older
        // integrations have also been observed sending 13-digit millisecond
        // values. To be robust to both, we disambiguate by magnitude: a value
        // past 10_000_000_000 (year ~2286 in seconds) is treated as ms and
        // divided by 1000; otherwise it is treated as seconds verbatim. Without
        // this, a production webhook sending seconds would always be flagged
        // stale (fail-closed DoS). See `ts_unit_to_secs`.
        let sig_header = super::webhook_util::header_str(&event.headers, "x-signature")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing x-signature header".into())
            })?;

        let mut ts: Option<&str> = None;
        let mut v1: Option<&str> = None;
        for part in sig_header.split(',') {
            let mut kv = part.splitn(2, '=');
            match kv.next().map(str::trim) {
                Some("ts") => ts = kv.next().map(str::trim),
                Some("v1") => v1 = kv.next().map(str::trim),
                _ => {}
            }
        }
        let ts =
            ts.ok_or_else(|| BillingError::WebhookVerification("x-signature missing ts=".into()))?;
        let v1 =
            v1.ok_or_else(|| BillingError::WebhookVerification("x-signature missing v1=".into()))?;

        // Replay protection. The `ts=` value's unit is ambiguous (see note
        // above): normalise to seconds before the freshness check.
        let ts_raw: i64 = ts.parse().map_err(|_| {
            BillingError::WebhookVerification("Mercado Pago ts not an integer".into())
        })?;
        let ts_secs = ts_unit_to_secs(ts_raw);
        if !super::webhook_util::timestamp_fresh(ts_secs, now_secs) {
            return Err(BillingError::WebhookVerification(format!(
                "Mercado Pago timestamp outside tolerance (ts_raw={ts_raw}, ts_secs={ts_secs})"
            )));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Official Mercado Pago signature scheme (V-CRIT-2): the tag is
        // HMAC-SHA256(secret, "id:{data.id};request-id:{x-request-id};ts:{ts};"),
        // where {data.id} comes from the webhook URL's `data.id` query param,
        // {x-request-id} from the request header, and {ts} from the `ts=` value
        // already parsed out of `x-signature`. The previous code HMAC'd
        // `ts ‖ raw_body`, which is structurally wrong and would never match a
        // genuine Mercado Pago signature.
        let query = event.query.as_deref().ok_or_else(|| {
            BillingError::WebhookVerification(
                "Mercado Pago webhook missing URL query string (data.id)".into(),
            )
        })?;
        let data_id = extract_query_param(query, "data.id").ok_or_else(|| {
            BillingError::WebhookVerification("Mercado Pago webhook query missing data.id".into())
        })?;
        // Per the official Mercado Pago spec the manifest's `data.id` MUST be
        // the LOWERCASED form when the value is alphanumeric: MP returns
        // data.id in UPPERCASE in the webhook query but SIGNS the lowercase
        // form ("if data.id is alphanumeric, it must be sent in lowercase").
        // Using the raw value verbatim would therefore reject every legitimate
        // webhook whose data.id contains uppercase letters (V-CRIT-2 residual
        // — confirmed against MP docs).
        let data_id_signed = if data_id.chars().all(|c| c.is_ascii_alphanumeric()) {
            data_id.to_ascii_lowercase()
        } else {
            data_id
        };
        let x_request_id = super::webhook_util::header_str(&event.headers, "x-request-id")
            .ok_or_else(|| {
                BillingError::WebhookVerification("Missing x-request-id header".into())
            })?;
        let manifest = format!(
            "id:{};request-id:{};ts:{};",
            data_id_signed, x_request_id, ts
        );
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.expose_secret().as_bytes(),
            manifest.as_bytes(),
            v1,
        ) {
            return Err(BillingError::WebhookVerification(
                "Mercado Pago signature mismatch".into(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Normalize Mercado Pago's native event taxonomy to the canonical
        // vocabulary the dispatch matches on (audit F#11 residual). A `payment`
        // event is the checkout-completion signal; a `preapproval` event is a
        // recurring-subscription lifecycle change.
        //
        // NOTE on id round-trip: MP keys its checkout intent by the preference
        // id (its `create_checkout` returns it as `session_id`), but a `payment`
        // webhook resource id (`data.id`) is the PAYMENT id — and MP payment
        // notifications are THIN (they carry only `{type, data:{id}}`); the
        // preference id and status are only obtainable by fetching the payment
        // resource via the MP API. So the checkout arm's intent recovery misses
        // and the dispatch refuses to grant (fail-closed, audit F#2/F#10).
        // Routing to the correct arm (instead of the old silent `_ =>` drop) is
        // the fix; enriching the webhook via an MP API fetch to recover the
        // preference id + confirm `status == "approved"` is the accepted
        // deferred enhancement. `preapproval` events carry the preapproval id
        // (which DOES equal the stored session_id), so the UPDATE arm processes
        // them normally.
        let native_event = data["type"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "payment" => super::provider::canonical::CHECKOUT_COMPLETED,
            "preapproval" => super::provider::canonical::SUBSCRIPTION_UPDATED,
            other => other,
        }
        .to_string();

        Ok(ParsedWebhook {
            event_type,
            customer_id: data["data"]["payer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            subscription_id: data["data"]["preapproval_id"]
                .as_str()
                .or_else(|| data["data"]["id"].as_str())
                .map(String::from),
            payment_id: data["data"]["id"].as_str().map(String::from),
            // Best-effort (see NOTE): `preapproval_id` round-trips to the stored
            // intent for preapproval events; for `payment` events the resource
            // is the payment id and won't match — the checkout arm refuses.
            checkout_session_id: data["data"]["preapproval_id"].as_str().map(String::from),
            // Mercado Pago preapprovals expose `next_payment_date` when set.
            current_period_end: super::provider::period_end_to_unix(
                data["data"].get("next_payment_date"),
            ),
            subscription_status: data["data"]["status"].as_str().map(String::from),
            user_id: data["data"]["external_reference"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            // MP amounts are decimal currency units; convert to minor units.
            amount_cents: data["data"]["transaction_amount"]
                .as_f64()
                .map(|f| (f * 100.0) as i64),
            currency: data["data"]["currency_id"].as_str().map(String::from),
            data,
        })
    }
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
        assert_eq!(provider.access_token.expose_secret(), "APP_USR-abc123");
        assert_eq!(provider.webhook_secret.expose_secret(), "whsec_def");
        assert_eq!(provider.base_url, "https://api.mercadopago.com");
    }

    #[test]
    fn test_mercado_pago_custom_base_url() {
        let provider = MercadoPagoProvider::new("token".into(), "wh".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }

    use crate::services::billing::webhook_util;

    /// Fixed values for the spec-correct Mercado Pago manifest (V-CRIT-2).
    /// The official scheme is:
    ///   HMAC-SHA256(secret, "id:{data.id};request-id:{x-request-id};ts:{ts};")
    /// where {data.id} is the URL query `data.id`, {x-request-id} is the
    /// request header, and {ts} is the `ts=` from `x-signature`.
    const TEST_DATA_ID: &str = "1234567890";
    const TEST_REQUEST_ID: &str = "abc-123";

    /// Sign a Mercado Pago webhook with the CORRECT manifest and return a
    /// `WebhookEvent` carrying the matching query string + x-request-id header.
    /// `ts_raw` is the literal value placed in both `x-signature`'s `ts=` and
    /// the manifest — it may be in seconds OR milliseconds; the verifier's
    /// `ts_unit_to_secs` normalises it. Per the MP spec the manifest is signed
    /// over the LOWERCASED alphanumeric `data.id`, even though the webhook
    /// query carries the ORIGINAL (often uppercase) value — so we sign the
    /// lowercased form here and put the original case in the query. (V-CRIT-2)
    fn signed_mp(payload: &[u8], ts_raw: i64, secret: &str, data_id: &str) -> WebhookEvent {
        let ts_str = ts_raw.to_string();
        let signed_data_id = if data_id.chars().all(|c| c.is_ascii_alphanumeric()) {
            data_id.to_ascii_lowercase()
        } else {
            data_id.to_string()
        };
        let manifest = format!("id:{signed_data_id};request-id:{TEST_REQUEST_ID};ts:{ts_str};");
        let v1 = webhook_util::hmac_sha256_hex(secret.as_bytes(), manifest.as_bytes());
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "x-signature",
            format!("ts={ts_str},v1={v1}").parse().unwrap(),
        );
        headers.insert("x-request-id", TEST_REQUEST_ID.parse().unwrap());
        WebhookEvent {
            provider: "mercado_pago".into(),
            payload: payload.to_vec(),
            headers,
            // Query carries the ORIGINAL case (MP returns uppercase); the
            // verifier lowercases an alphanumeric value before rebuilding the
            // manifest, so it matches the lowercased signature above.
            query: Some(format!("data.id={data_id}")),
        }
    }

    /// Native Mercado Pago events must normalize to the canonical vocabulary
    /// the provider-agnostic dispatch matches on (audit F#11). NOTE: a `payment`
    /// webhook is THIN and does not round-trip the stored preference id, so the
    /// checkout arm fails closed on intent recovery — but the event still
    /// reaches the correct arm (not the old silent `_ =>` drop).
    #[tokio::test]
    async fn verify_webhook_normalizes_native_events_to_canonical() {
        let provider = MercadoPagoProvider::new("token".into(), "whsec".into());
        // Pin "now" to a fixed point so the test never depends on the wall
        // clock (V-CRIT-2 residuals). The signed ts is in milliseconds.
        let now_ms: i64 = 1_700_000_000_000;
        let now_secs = now_ms / 1000;

        let cases: &[(&str, &str)] = &[
            // Thin payment notification (the typical MP shape).
            (
                r#"{"type":"payment","data":{"id":"pay_1"}}"#,
                "checkout.session.completed",
            ),
            // A preapproval lifecycle event carries the preapproval id (= stored
            // session_id), so the UPDATE arm can find the row.
            (
                r#"{"type":"preapproval","data":{"id":"preap_1","status":"authorized","preapproval_id":"preap_1","next_payment_date":"2026-12-31T00:00:00Z","external_reference":"42","transaction_amount":99.9,"currency_id":"BRL"}}"#,
                "customer.subscription.updated",
            ),
            // Unmapped → passthrough.
            (
                r#"{"type":"merchant_order","data":{"id":"mo_1"}}"#,
                "merchant_order",
            ),
        ];
        for (body, expected) in cases {
            // The URL query data.id is the canonical manifest source (V-CRIT-2);
            // it is independent of the body's resource id.
            let evt = signed_mp(body.as_bytes(), now_ms, "whsec", TEST_DATA_ID);
            let parsed = provider
                .verify_webhook_at(evt, now_secs)
                .await
                .expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // Structured fields on a preapproval (rich) event.
        let evt = signed_mp(
            br#"{"type":"preapproval","data":{"id":"preap_1","status":"authorized","preapproval_id":"preap_1","next_payment_date":"2026-12-31T00:00:00Z","external_reference":"42","transaction_amount":99.9,"currency_id":"BRL"}}"#,
            now_ms,
            "whsec",
            TEST_DATA_ID,
        );
        let parsed = provider.verify_webhook_at(evt, now_secs).await.unwrap();
        assert_eq!(parsed.subscription_id.as_deref(), Some("preap_1"));
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("preap_1"));
        assert_eq!(parsed.subscription_status.as_deref(), Some("authorized"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(9990));
        assert_eq!(parsed.currency.as_deref(), Some("BRL"));
    }

    /// V-CRIT-2: the verifier MUST authenticate the official Mercado Pago
    /// manifest `id:{data.id};request-id:{x-request-id};ts:{ts};` (built from
    /// the URL query data.id + x-request-id header + x-signature ts), NOT the
    /// legacy `ts ‖ body`. These cases pin fixed values so the test asserts the
    /// spec, not self-consistency with the old wrong format.
    #[tokio::test]
    async fn verify_webhook_uses_official_manifest() {
        let provider = MercadoPagoProvider::new("token".into(), "whsec".into());
        // Pin "now" so the freshness window never depends on the wall clock
        // (V-CRIT-2 residuals). The signed ts is in milliseconds.
        let now_ms: i64 = 1_700_000_000_000;
        let now_secs = now_ms / 1000;
        let body = br#"{"type":"payment","data":{"id":"1234567890"}}"#;

        // Positive: manifest signed exactly as the spec dictates verifies.
        let evt = signed_mp(body, now_ms, "whsec", TEST_DATA_ID);
        provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect("spec-correct manifest must verify");

        // Negative: a tag computed over the LEGACY `ts ‖ body` (old wrong
        // scheme) must NOT verify — proves we are not silently still using it.
        let ts_str = now_ms.to_string();
        let mut legacy = Vec::with_capacity(ts_str.len() + body.len());
        legacy.extend_from_slice(ts_str.as_bytes());
        legacy.extend_from_slice(body);
        let legacy_v1 = webhook_util::hmac_sha256_hex(b"whsec", &legacy);
        let mut h = axum::http::HeaderMap::new();
        h.insert(
            "x-signature",
            format!("ts={ts_str},v1={legacy_v1}").parse().unwrap(),
        );
        h.insert("x-request-id", TEST_REQUEST_ID.parse().unwrap());
        let evt = WebhookEvent {
            provider: "mercado_pago".into(),
            payload: body.to_vec(),
            headers: h,
            query: Some(format!("data.id={TEST_DATA_ID}")),
        };
        let err = provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect_err("legacy manifest must be rejected");
        assert!(err.to_string().to_lowercase().contains("mismatch"));

        // Negative: missing query string → fail closed.
        let mut evt = signed_mp(body, now_ms, "whsec", TEST_DATA_ID);
        evt.query = None;
        provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect_err("missing query must fail closed");

        // Negative: missing x-request-id header → fail closed.
        let mut evt = signed_mp(body, now_ms, "whsec", TEST_DATA_ID);
        evt.headers.remove("x-request-id");
        provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect_err("missing x-request-id must fail closed");

        // Negative: query data.id differs from signed manifest → fail closed.
        let mut evt = signed_mp(body, now_ms, "whsec", TEST_DATA_ID);
        evt.query = Some("data.id=0000000000".into());
        provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect_err("data.id mismatch must fail closed");
    }

    /// V-CRIT-2 residual (ts UNIT): Mercado Pago's documentation example uses
    /// a 10-digit SECONDS ts (ts=1704908010 ≈ 2024-01-10 16:13:30Z). A
    /// seconds-shaped ts MUST verify (the old code divided by 1000 unconditionally
    /// and would flag every seconds-based webhook as stale — a fail-closed DoS).
    /// "now" is pinned to within 300s of the doc ts so the test is deterministic
    /// and does not call `Utc::now()` on the assertion path.
    #[tokio::test]
    async fn verify_webhook_accepts_seconds_shaped_ts() {
        let provider = MercadoPagoProvider::new("token".into(), "whsec".into());
        // The literal value from Mercado Pago's webhook docs.
        const TS_SECS: i64 = 1_704_908_010;
        // Pin "now" within the 300s tolerance (MAX_SKEW_SECS = 300) of the doc ts.
        // 60s after the doc ts — comfortably inside the window and deterministic.
        let now_secs = TS_SECS + 60;
        assert!(
            (now_secs - TS_SECS).abs() <= 300,
            "now must be within 300s of the doc ts for a deterministic freshness window"
        );
        let body = br#"{"type":"payment","data":{"id":"1234567890"}}"#;

        let evt = signed_mp(body, TS_SECS, "whsec", TEST_DATA_ID);
        provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect("seconds-shaped ts (doc value) must verify");

        // Sanity: a seconds-shaped ts that is genuinely stale must still be
        // rejected (the disambiguation must not silently widen the window).
        let stale_now = TS_SECS + 300 + 1; // one second past the tolerance
        let evt = signed_mp(body, TS_SECS, "whsec", TEST_DATA_ID);
        let err = provider
            .verify_webhook_at(evt, stale_now)
            .await
            .expect_err("stale seconds-shaped ts must be rejected");
        assert!(
            err.to_string().to_lowercase().contains("outside tolerance"),
            "stale ts should be rejected as outside tolerance, got: {err}"
        );
    }

    /// V-CRIT-2 residual (data.id LOWERCASING, per the MP spec): Mercado Pago
    /// returns `data.id` in UPPERCASE in the webhook query but SIGNS the
    /// LOWERCASED form. The verifier must lowercase an alphanumeric query value
    /// before building the manifest; using it verbatim would reject every
    /// legitimate webhook whose data.id contains uppercase letters (the exact
    /// regression this guards — verified against the MP docs).
    #[tokio::test]
    async fn verify_webhook_lowercases_alphanumeric_data_id() {
        let provider = MercadoPagoProvider::new("token".into(), "whsec".into());
        let now_ms: i64 = 1_700_000_000_000;
        let now_secs = now_ms / 1000;
        // A purely-alphanumeric UPPERCASE data.id (the shape MP uppercases).
        const UPPER_DATA_ID: &str = "PAYIDABC123";
        let body = br#"{"type":"payment","data":{"id":"PAYIDABC123"}}"#;

        // signed_mp signs the LOWERCASED form but puts the UPPERCASE value in
        // the query — mirroring MP. The verifier must lowercase the query to
        // match the signed manifest.
        let evt = signed_mp(body, now_ms, "whsec", UPPER_DATA_ID);
        assert_eq!(evt.query.as_deref(), Some("data.id=PAYIDABC123"));
        provider
            .verify_webhook_at(evt, now_secs)
            .await
            .expect("verifier must lowercase the uppercase data.id query to match MP's lowercased signature");

        // Negative: an unrelated data.id in the query must fail closed even
        // after lowercasing (the verifier cannot invent a match).
        let mut evt2 = signed_mp(body, now_ms, "whsec", UPPER_DATA_ID);
        evt2.query = Some("data.id=ZZZ999".into());
        provider
            .verify_webhook_at(evt2, now_secs)
            .await
            .expect_err("an unrelated data.id must fail closed");
    }

    /// `ts_unit_to_secs` collapses milliseconds to seconds and leaves seconds
    /// untouched (V-CRIT-2 residual). The threshold is 10_000_000_000.
    #[test]
    fn ts_unit_to_secs_disambiguates_by_magnitude() {
        // The MP doc example (seconds) — left verbatim.
        assert_eq!(ts_unit_to_secs(1_704_908_010), 1_704_908_010);
        // A 13-digit millisecond value — divided by 1000.
        assert_eq!(ts_unit_to_secs(1_704_908_010_000), 1_704_908_010);
        // Exactly at the threshold (10_000_000_000) is treated as seconds.
        assert_eq!(ts_unit_to_secs(10_000_000_000), 10_000_000_000);
        // One past the threshold is treated as milliseconds.
        assert_eq!(ts_unit_to_secs(10_000_000_001), 10_000_000);
        // Zero and negatives pass through unchanged.
        assert_eq!(ts_unit_to_secs(0), 0);
        assert_eq!(ts_unit_to_secs(-1), -1);
    }

    /// `extract_query_param` parses `data.id` out of a realistic MP query
    /// string, including URL-encoded values (V-CRIT-2).
    #[test]
    fn extract_query_param_finds_data_id() {
        assert_eq!(
            extract_query_param("data.id=1234567890&type=payment", "data.id").as_deref(),
            Some("1234567890")
        );
        // Leading `?` from a full query string should be tolerated (the pair
        // starting with `?data.id` won't match `data.id` — the caller strips
        // any leading `?`; here we just confirm a plain match works).
        assert_eq!(
            extract_query_param("data.id=abc", "data.id").as_deref(),
            Some("abc")
        );
        // Percent-decoded + space-unescaped value.
        assert_eq!(
            extract_query_param("data.id=foo%20bar%2Bbaz&type=payment", "data.id").as_deref(),
            Some("foo bar+baz")
        );
        assert!(extract_query_param("type=payment", "data.id").is_none());
    }
}
