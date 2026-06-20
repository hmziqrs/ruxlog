//! PayPal Commerce billing provider integration (Global).
//!
//! Supports PayPal, Venmo, credit/debit cards, and subscriptions.

use async_trait::async_trait;

use super::provider::{
    BillingError, BillingProvider, CheckoutSession, ParsedWebhook, SubscriptionInfo, WebhookEvent,
};

// V-MED-10: every outbound PayPal call goes through this client (built once in
// `new` with timeouts, or overridden via `with_http_client` with the shared
// AppState client). Never a bare `reqwest::Client::new()`.
use crate::state::build_http_client;

/// PayPal Commerce billing provider.
pub struct PayPalProvider {
    pub client_id: String,
    pub client_secret: String,
    pub webhook_secret: String,
    /// PayPal webhook ID (the identifier PayPal issues when you register the
    /// webhook). Required by the verify-webhook-signature API. `None` ⇒
    /// verification fails closed. (PayPal does not use a shared HMAC secret; the
    /// legacy `webhook_secret` field is retained only for constructor
    /// compatibility and is not used for verification.)
    pub webhook_id: Option<String>,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl PayPalProvider {
    pub fn new(client_id: String, client_secret: String, webhook_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            webhook_secret,
            webhook_id: None,
            // Production by default; override with the sandbox host via
            // PAYPAL_API_BASE_URL for development. See plan Phase 6f.
            base_url: std::env::var("PAYPAL_API_BASE_URL")
                .unwrap_or_else(|_| "https://api-m.paypal.com".to_string()),
            http_client: build_http_client(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Set the PayPal webhook ID (required to verify webhooks).
    pub fn with_webhook_id(mut self, webhook_id: String) -> Self {
        self.webhook_id = Some(webhook_id);
        self
    }

    /// V-MED-10: inject the shared, timeout-configured client from `AppState`.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }
}

impl PayPalProvider {
    /// Get an OAuth access token from PayPal.
    async fn get_access_token(&self) -> Result<String, BillingError> {
        let client = self.http_client.clone();
        let resp = client
            .post(format!("{}/v1/oauth2/token", self.base_url))
            .header("Accept", "application/json")
            .header("Accept-Language", "en_US")
            .form(&[("grant_type", "client_credentials")])
            .basic_auth(&self.client_id, Some(&self.client_secret))
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

        data["access_token"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| BillingError::ProviderApi("No access_token in response".to_string()))
    }

    /// Best-effort fetch of a billing subscription's next charge time
    /// (`billing_info.next_billing_time`, RFC 3339) as a Unix timestamp. Used on
    /// `PAYMENT.SALE.COMPLETED` / `PAYMENT.CAPTURE.COMPLETED`, whose `resource` is
    /// the SALE (no billing info of its own), to obtain the linked subscription's
    /// authoritative next billing time so the dispatch can refresh the row's
    /// `current_period_end` and keep the renewing subscriber admitted across
    /// cycles. Returns `None` on any failure (auth, network, non-2xx, missing
    /// field) → fail-closed (audit F#11 round-2).
    async fn fetch_subscription_period_end(&self, subscription_id: &str) -> Option<i64> {
        let token = self.get_access_token().await.ok()?;
        let client = self.http_client.clone();
        let url = format!(
            "{}/v1/billing/subscriptions/{}",
            self.base_url, subscription_id
        );
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let data: serde_json::Value = resp.json().await.ok()?;
        super::provider::period_end_to_unix(
            data.get("billing_info")
                .and_then(|b| b.get("next_billing_time")),
        )
    }
}

#[async_trait]
impl BillingProvider for PayPalProvider {
    fn provider_name(&self) -> &'static str {
        "paypal"
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

        // Create a REAL PayPal billing subscription (not a one-time order) so the
        // `session_id` we store — and key the checkout intent by — is a
        // subscription id (`I-…`). `BILLING.SUBSCRIPTION.ACTIVATED` echoes that
        // same id back as `resource.id`, so the checkout arm's intent recovery
        // connects (audit F#11 residual; the prior Orders flow keyed the intent
        // on `ORDER-…` while the activation webhook emitted `I-…`, so every
        // subscription checkout silently never granted). A subscription also
        // carries an authoritative `billing_info.next_billing_time`, giving the
        // grant a real period end so the paywall admits the paying subscriber.
        //
        // `plan_slug` is the provider-side PayPal plan id (pricing/cycle live in
        // PayPal). `custom_id` round-trips the user id for diagnostics; the
        // grant itself is driven by the server-bound checkout intent.
        let body = serde_json::json!({
            "plan_id": plan_slug,
            "custom_id": user_id.to_string(),
            "subscriber": { "email_address": customer_email },
            "application_context": {
                "brand_name": "Ruxlog",
                "user_action": "SUBSCRIBE_NOW",
                "return_url": success_url,
                "cancel_url": cancel_url,
                "shipping_preference": "NO_SHIPPING",
            },
        });

        let resp = client
            .post(format!("{}/v1/billing/subscriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("PayPal-Request-Id", uuid::Uuid::new_v4().to_string())
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

        // Extract the approval link from the response.
        let checkout_url = data["links"]
            .as_array()
            .and_then(|links| {
                links
                    .iter()
                    .find(|link| link["rel"].as_str() == Some("approve"))
                    .and_then(|link| link["href"].as_str().map(String::from))
            })
            .unwrap_or_default();

        Ok(CheckoutSession {
            session_id: data["id"].as_str().unwrap_or_default().to_string(),
            checkout_url,
        })
    }

    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError> {
        let token = self.get_access_token().await?;
        let client = self.http_client.clone();
        let url = format!(
            "{}/v1/billing/subscriptions/{}/cancel",
            self.base_url, provider_subscription_id
        );

        let reason = if immediately {
            "Cancelled immediately by admin"
        } else {
            "Cancelled at end of billing cycle"
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "reason": reason }))
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
            "{}/v1/billing/subscriptions/{}",
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

        let current_end = data["billing_info"]["next_billing_time"]
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
        // PayPal does NOT sign webhooks with a shared HMAC secret — the previous
        // local HMAC could never match a real PayPal signature, so verification
        // was effectively absent. Instead we call PayPal's
        // verify-webhook-signature API, which checks the cert-backed signature
        // server-side, and trust ONLY `verification_status == "SUCCESS"`.
        // (Extra latency + a PayPal-side dependency; the access token is fetched
        // per call — caching is a future optimization.)
        let webhook_id = self.webhook_id.as_ref().ok_or_else(|| {
            BillingError::WebhookVerification(
                "PAYPAL_WEBHOOK_ID not configured; cannot verify PayPal webhook".into(),
            )
        })?;

        let header = |name: &str| {
            super::webhook_util::header_str(&event.headers, name).unwrap_or_default()
        };
        let transmission_id = header("PAYPAL-TRANSMISSION-ID");
        let transmission_time = header("PAYPAL-TRANSMISSION-TIME");
        let cert_url = header("PAYPAL-CERT-URL");
        let auth_algo = header("PAYPAL-AUTH-ALGO");
        let transmission_sig = header("PAYPAL-TRANSMISSION-SIG");

        if transmission_id.is_empty()
            || transmission_time.is_empty()
            || cert_url.is_empty()
            || auth_algo.is_empty()
            || transmission_sig.is_empty()
        {
            return Err(BillingError::WebhookVerification(
                "PayPal webhook missing required transmission headers".into(),
            ));
        }

        let payload_str = std::str::from_utf8(&event.payload)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        let webhook_event: serde_json::Value = serde_json::from_str(payload_str)
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;

        let access_token = self.get_access_token().await?;

        let verify_body = serde_json::json!({
            "transmission_id": transmission_id,
            "transmission_time": transmission_time,
            "cert_url": cert_url,
            "auth_algo": auth_algo,
            "transmission_sig": transmission_sig,
            "webhook_id": webhook_id,
            "webhook_event": webhook_event,
        });

        let client = self.http_client.clone();
        let resp = client
            .post(format!(
                "{}/v1/notifications/verify-webhook-signature",
                self.base_url
            ))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&verify_body)
            .send()
            .await
            .map_err(|e| BillingError::ProviderApi(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(BillingError::WebhookVerification(format!(
                "PayPal verify-webhook-signature call failed: {body}"
            )));
        }

        let vdata: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BillingError::WebhookVerification(e.to_string()))?;
        let status = vdata["verification_status"].as_str().unwrap_or("");
        if status != "SUCCESS" {
            return Err(BillingError::WebhookVerification(format!(
                "PayPal webhook verification_status={status}"
            )));
        }

        // Normalize PayPal's native event taxonomy to the canonical vocabulary
        // the dispatch matches on (audit F#11 residual).
        let native_event = webhook_event["event_type"].as_str().unwrap_or_default();
        let event_type = match native_event {
            "BILLING.SUBSCRIPTION.ACTIVATED" => super::provider::canonical::CHECKOUT_COMPLETED,
            "BILLING.SUBSCRIPTION.CANCELLED" => super::provider::canonical::SUBSCRIPTION_DELETED,
            "BILLING.SUBSCRIPTION.UPDATED"
            | "BILLING.SUBSCRIPTION.EXPIRED"
            | "BILLING.SUBSCRIPTION.SUSPENDED" => super::provider::canonical::SUBSCRIPTION_UPDATED,
            "PAYMENT.SALE.COMPLETED" | "PAYMENT.CAPTURE.COMPLETED" => {
                super::provider::canonical::PAYMENT_SUCCEEDED
            }
            other => other,
        }
        .to_string();
        let resource = &webhook_event["resource"];
        let resource_id = resource["id"].as_str().map(String::from);
        let billing_agreement_id = resource["billing_agreement_id"]
            .as_str()
            .map(String::from);

        // For a SALE/CAPTURE payment the `resource` IS the sale, so `resource.id`
        // is the SALE id (`S-…`) — NOT the subscription. The linked recurring
        // subscription lives in `billing_agreement_id` (`I-…`). Using the SALE id
        // as `subscription_id` meant the dispatch's subscription lookup never
        // matched a row, so a renewal recorded no owner AND never refreshed the
        // row's `current_period_end` → after the first cycle the paywall denied
        // the renewing subscriber (audit F#11 round-2). For lifecycle events
        // (BILLING.SUBSCRIPTION.*) the `resource` IS the subscription, so
        // `resource.id` is the correct subscription id.
        let is_sale = matches!(
            native_event,
            "PAYMENT.SALE.COMPLETED" | "PAYMENT.CAPTURE.COMPLETED"
        );
        let subscription_id = if is_sale {
            billing_agreement_id.clone()
        } else {
            resource_id.clone()
        };

        // Resolve the billing period end. Lifecycle/activation events carry
        // `billing_info.next_billing_time` inline; a SALE payment does NOT, so
        // fetch the linked subscription (by `billing_agreement_id`) for the
        // authoritative next billing time. The dispatch's PAYMENT_SUCCEEDED arm
        // refreshes the row's `current_period_end` from this so a renewing
        // subscriber stays admitted across cycles (audit F#11 round-2). Fetch
        // failures degrade to None (fail-closed).
        let inline_period_end = super::provider::period_end_to_unix(
            resource
                .get("billing_info")
                .and_then(|b| b.get("next_billing_time")),
        );
        let current_period_end = match inline_period_end {
            Some(ts) => Some(ts),
            None => match subscription_id.as_deref() {
                Some(sub_id) if !sub_id.is_empty() => {
                    self.fetch_subscription_period_end(sub_id).await
                }
                _ => None,
            },
        };

        Ok(ParsedWebhook {
            event_type,
            customer_id: resource["subscriber"]["payer_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            // The recurring subscription reference. Lifecycle events → the
            // subscription id (`resource.id`); SALE/CAPTURE → `billing_agreement_id`.
            subscription_id,
            // The SALE/payment id dedups the payment row (history).
            payment_id: resource_id.clone(),
            // Lifecycle events key the checkout intent by the subscription id
            // (`resource.id`); SALE payments never reach the checkout arm, so this
            // value is irrelevant for them (kept as the resource id for
            // diagnostics only).
            checkout_session_id: resource_id,
            current_period_end,
            // PayPal status (ACTIVE/CANCELLED/SUSPENDED/EXPIRED…) folds to our
            // vocabulary via canonical_subscription_status.
            subscription_status: resource["status"].as_str().map(String::from),
            user_id: resource["custom_id"]
                .as_str()
                .and_then(|s| s.parse().ok()),
            amount_cents: resource["amount"]["total"]
                .as_str()
                .and_then(|s| {
                    // PayPal money is a decimal string; store as minor units.
                    s.parse::<f64>().ok().map(|f| (f * 100.0) as i64)
                })
                .or_else(|| resource["amount"]["value"].as_str().and_then(|s| s.parse().ok())),
            currency: resource["amount"]["currency_code"]
                .as_str()
                .map(String::from),
            data: webhook_event,
        })
    }

    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        // PayPal doesn't have a native billing portal — redirect to PayPal settings.
        // Use the sandbox customer portal only when the API base points at sandbox
        // (i.e. dev override); otherwise the production portal. See plan Phase 6f.
        let portal_base = if self.base_url.contains("sandbox") {
            "https://www.sandbox.paypal.com"
        } else {
            "https://www.paypal.com"
        };
        Ok(format!(
            "{portal_base}/myaccount/autopay/connect/{}?return_url={}",
            provider_customer_id,
            urlencoding::encode(return_url)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paypal_provider_name() {
        let provider = PayPalProvider::new("cid".into(), "secret".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "paypal");
    }

    #[test]
    fn test_paypal_new() {
        let provider = PayPalProvider::new(
            "AWx_test_client_id".into(),
            "test_secret_key".into(),
            "whsec_test".into(),
        );
        assert_eq!(provider.client_id, "AWx_test_client_id");
        assert_eq!(provider.client_secret, "test_secret_key");
        assert_eq!(provider.webhook_secret, "whsec_test");
        assert_eq!(provider.base_url, "https://api-m.paypal.com");
    }

    #[test]
    fn test_paypal_custom_base_url() {
        let provider = PayPalProvider::new("c".into(), "s".into(), "w".into())
            .with_base_url("http://localhost:9999".into());
        assert_eq!(provider.base_url, "http://localhost:9999");
    }
}
