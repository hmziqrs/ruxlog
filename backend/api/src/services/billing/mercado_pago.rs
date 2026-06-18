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
            // Production by default; override with the sandbox host via
            // MERCADO_PAGO_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("MERCADO_PAGO_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.mercadopago.com".to_string()),
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
        // Mercado Pago signs with x-signature: "ts=<ms>,v1=<hmac-sha256>".
        // The tag is HMAC-SHA256(secret, "{ts}{raw_body}"). ts is in ms.
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
        let ts = ts.ok_or_else(|| {
            BillingError::WebhookVerification("x-signature missing ts=".into())
        })?;
        let v1 = v1.ok_or_else(|| {
            BillingError::WebhookVerification("x-signature missing v1=".into())
        })?;

        // Replay protection (ts is in milliseconds).
        let ts_ms: i64 = ts.parse().map_err(|_| {
            BillingError::WebhookVerification("Mercado Pago ts not an integer".into())
        })?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        if !super::webhook_util::timestamp_fresh(ts_ms / 1000, now_ms / 1000) {
            return Err(BillingError::WebhookVerification(format!(
                "Mercado Pago timestamp outside tolerance (ts_ms={ts_ms})"
            )));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        // Verify HMAC over "{ts}{raw_body}" in constant time.
        let mut manifest = Vec::with_capacity(ts.len() + event.payload.len());
        manifest.extend_from_slice(ts.as_bytes());
        manifest.extend_from_slice(&event.payload);
        if !super::webhook_util::verify_hmac_sha256_hex(
            self.webhook_secret.as_bytes(),
            &manifest,
            v1,
        ) {
            return Err(BillingError::WebhookVerification(
                "Mercado Pago signature mismatch".into(),
            ));
        }

        let data: serde_json::Value = serde_json::from_str(&payload_str)
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
            current_period_end: super::provider::period_end_to_unix(data["data"].get("next_payment_date")),
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

    use crate::services::billing::webhook_util;

    /// Sign a Mercado Pago webhook: `x-signature: ts={ms},v1={hmac_hex}`,
    /// where the tag is HMAC-SHA256(secret, "{ts}{body}") and ts is in ms.
    fn signed_mp(payload: &[u8], ts_ms: i64, secret: &str) -> WebhookEvent {
        let ts_str = ts_ms.to_string();
        let mut msg = Vec::with_capacity(ts_str.len() + payload.len());
        msg.extend_from_slice(ts_str.as_bytes());
        msg.extend_from_slice(payload);
        let v1 = webhook_util::hmac_sha256_hex(secret.as_bytes(), &msg);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "x-signature",
            format!("ts={ts_str},v1={v1}").parse().unwrap(),
        );
        WebhookEvent {
            provider: "mercado_pago".into(),
            payload: payload.to_vec(),
            headers,
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
        let now_ms = chrono::Utc::now().timestamp_millis();

        let cases: &[(&str, &str)] = &[
            // Thin payment notification (the typical MP shape).
            (r#"{"type":"payment","data":{"id":"pay_1"}}"#, "checkout.session.completed"),
            // A preapproval lifecycle event carries the preapproval id (= stored
            // session_id), so the UPDATE arm can find the row.
            (
                r#"{"type":"preapproval","data":{"id":"preap_1","status":"authorized","preapproval_id":"preap_1","next_payment_date":"2026-12-31T00:00:00Z","external_reference":"42","transaction_amount":99.9,"currency_id":"BRL"}}"#,
                "customer.subscription.updated",
            ),
            // Unmapped → passthrough.
            (r#"{"type":"merchant_order","data":{"id":"mo_1"}}"#, "merchant_order"),
        ];
        for (body, expected) in cases {
            let evt = signed_mp(body.as_bytes(), now_ms, "whsec");
            let parsed = provider.verify_webhook(evt).await.expect("must verify");
            assert_eq!(parsed.event_type, *expected, "body={body}");
        }

        // Structured fields on a preapproval (rich) event.
        let evt = signed_mp(
            br#"{"type":"preapproval","data":{"id":"preap_1","status":"authorized","preapproval_id":"preap_1","next_payment_date":"2026-12-31T00:00:00Z","external_reference":"42","transaction_amount":99.9,"currency_id":"BRL"}}"#,
            now_ms,
            "whsec",
        );
        let parsed = provider.verify_webhook(evt).await.unwrap();
        assert_eq!(parsed.subscription_id.as_deref(), Some("preap_1"));
        assert_eq!(parsed.checkout_session_id.as_deref(), Some("preap_1"));
        assert_eq!(parsed.subscription_status.as_deref(), Some("authorized"));
        assert_eq!(parsed.user_id, Some(42));
        assert_eq!(parsed.amount_cents, Some(9990));
        assert_eq!(parsed.currency.as_deref(), Some("BRL"));
    }
}
