//! Generic billing provider trait.
//!
//! Every payment integration (Stripe, Polar.sh, LemonSqueezy, Paddle, Crypto)
//! implements this trait. The admin panel and checkout flows work through this
//! abstraction, so adding a new provider requires only implementing this trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Result of creating a checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    /// Provider-specific session ID
    pub session_id: String,
    /// URL the customer should be redirected to
    pub checkout_url: String,
}

/// Result of a subscription lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub provider_subscription_id: String,
    pub status: String,
    pub current_period_end: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub cancel_at_period_end: bool,
}

/// Result of a payment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub provider_payment_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
}

/// Incoming webhook event from a provider.
///
/// Carries the **raw** request body and the **full** header set, because every
/// provider signs differently: Stripe/Paddle/Airwallex read timestamp+signature
/// pairs, PayPal needs five headers for its verify API, Mercado Pago uses
/// `x-signature`+`x-request-id`. The previous single-`signature` shape forced
/// the controller to guess one header per provider and dropped the rest, so
/// providers that needed a timestamp header could never verify. See plan 1a.
#[derive(Debug, Clone)]
pub struct WebhookEvent {
    /// Provider that sent this event
    pub provider: String,
    /// Raw payload bytes (exactly as received — never re-encoded JSON)
    pub payload: Vec<u8>,
    /// All request headers; each provider reads the ones it needs.
    pub headers: axum::http::HeaderMap,
    /// The raw URL query string of the incoming webhook request.
    /// Mercado Pago's signature scheme signs over `data.id` taken from the
    /// webhook URL's query string (not the body), so the receiver must forward
    /// it. `None` for providers/tests that don't use it. (V-CRIT-2)
    pub query: Option<String>,
}

/// Canonical webhook event vocabulary.
///
/// The dispatch (`process_webhook_event`) matches `event_type` against exactly
/// these literals. Every provider's `verify_webhook` MUST translate its native
/// event to one of these — otherwise the event falls to the log-only `_ =>` arm
/// and is silently dropped, which is the cross-cluster defect that previously
/// defeated the paywall for all 8 non-Stripe providers (audit F#11 residual):
/// their native event types (`subscription.activated`, `BILLING.SUBSCRIPTION.*`,
/// `order_created`, …) never matched the Stripe-style literals the dispatch
/// keys on. The canonical set mirrors Stripe so the existing dispatch arms are
/// unchanged; normalization happens at the provider boundary, where each
/// provider's event taxonomy is known.
pub mod canonical {
    /// A checkout the customer initiated has been paid/completed → GRANT (a
    /// `post_purchases` row for a one-time purchase, or a `subscriptions` row
    /// with status Active + `current_period_end` for a subscription).
    pub const CHECKOUT_COMPLETED: &str = "checkout.session.completed";
    /// An EXISTING subscription's state changed (renewal, status transition) →
    /// UPDATE the subscription row's status + `current_period_end`.
    pub const SUBSCRIPTION_UPDATED: &str = "customer.subscription.updated";
    /// An existing subscription was cancelled → mark the row Canceled.
    pub const SUBSCRIPTION_DELETED: &str = "customer.subscription.deleted";
    /// A recurring/one-time payment succeeded → record a `payments` row
    /// (history; not itself a grant — the grant happens on CHECKOUT_COMPLETED).
    pub const PAYMENT_SUCCEEDED: &str = "invoice.payment_succeeded";
    /// Crypto one-time payment confirmed (on-chain). Reachable only if the
    /// crypto provider ever stops fail-closing in `verify_webhook`.
    pub const PAYMENT_CONFIRMED: &str = "payment.confirmed";
    /// Crypto one-time payment detected but pending confirmations.
    pub const PAYMENT_PENDING: &str = "payment.pending";
}

/// Normalize a provider-native subscription status to our canonical vocabulary
/// (`active` | `trialing` | `past_due` | `canceled` | `expired` — the values of
/// [`SubscriptionStatus`]). Returns `None` for anything unrecognized, in which
/// case the dispatch leaves the row's existing status untouched (safe — the
/// paywall keeps gating on whatever status was last persisted, plus the
/// authoritative `current_period_end`). This is the only place provider status
/// strings are trusted; it folds the vocabularies of all 9 providers into one.
pub fn canonical_subscription_status(raw: Option<&str>) -> Option<&'static str> {
    let s = raw?.trim().to_ascii_lowercase();
    Some(match s.as_str() {
        // active
        "active"
        | "activated"
        | "subscription_active"
        | "incomplete_active"
        | "running"
        | "authorized" => "active",
        // trialing
        "trialing" | "trialling" | "trial" | "in_trial" | "pending_trial" | "on_trial" => {
            "trialing"
        }
        // past_due / problem (revoke access but not terminal)
        "past_due" | "pastdue" | "unpaid" | "problem" | "suspended" | "paused" | "on_hold"
        | "incomplete" => "past_due",
        // canceled
        "canceled" | "cancelled" | "subscription_cancelled" | "revoked" => "canceled",
        // expired / terminal-revoke. `completed` (Razorpay: subscription ran its
        // full cycle) and `ended` (Mercado Pago preapproval terminated) are
        // terminal — no further access (audit F#11 residual).
        "expired" | "halted" | "failed" | "completed" | "ended" => "expired",
        _ => return None,
    })
}

/// Parsed webhook after verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedWebhook {
    /// **Canonical** event type — one of [`canonical`] constants. Each
    /// provider's `verify_webhook` translates its native event to this so the
    /// provider-agnostic dispatch can match it.
    pub event_type: String,
    /// Provider customer ID
    pub customer_id: String,
    /// Provider subscription ID (if subscription event)
    pub subscription_id: Option<String>,
    /// Provider payment ID (if payment event)
    pub payment_id: Option<String>,
    /// When the current billing period ends, for subscription events that carry
    /// it (audit F#11). Persisted onto the `subscriptions` row so the paywall
    /// (`user_has_active_subscription`) can deny access once it lapses instead
    /// of trusting a stale `status` forever. `None` when the provider/event
    /// doesn't supply one — in which case the paywall fails closed.
    pub current_period_end: Option<i64>,
    /// The provider id that keys the server-bound checkout intent — i.e. the
    /// value the provider returned as `session_id` from `create_checkout`. The
    /// CHECKOUT_COMPLETED dispatch arm recovers the (single-use, atomic) intent
    /// with this id. After the F#11 residual fixes every provider populates
    /// this with the exact round-trip id; the dispatch still falls back to
    /// `subscription_id` / `payment_id` as defense-in-depth. LemonSqueezy is
    /// the one known miss: its webhook resource id is the order/subscription id
    /// (not the stored checkout id), so neither this nor the fallbacks match and
    /// the grant is refused (fail-closed, accepted deferral). Grant is
    /// impossible without the intent (audit F#2/F#3), so an unrecoverable id
    /// safely denies.
    pub checkout_session_id: Option<String>,
    /// **Canonical** subscription status for lifecycle events
    /// (see [`canonical_subscription_status`]). Drives the subscription-row
    /// status update. `None` ⇒ leave the row's existing status untouched.
    pub subscription_status: Option<String>,
    /// Payer id extracted from the provider's metadata/reference. Diagnostic
    /// for the CHECKOUT_COMPLETED grant path (which uses the server-bound
    /// intent's `user_id`); the PAYMENT_SUCCEEDED arm uses it to attribute a
    /// `payments` row, falling back to a subscription lookup when absent.
    pub user_id: Option<i32>,
    /// Payment amount in minor units (cents) and ISO currency, for the
    /// payment-record arm. `None` ⇒ the dispatch records `0` / `"usd"`.
    pub amount_cents: Option<i64>,
    pub currency: Option<String>,
    /// Raw verified event data as JSON (kept for the metadata blob stored on
    /// rows). The dispatch MUST NOT read provider-specific JSON paths from this
    /// — all dispatch-relevant facts are in the structured fields above so the
    /// path is provider-agnostic.
    pub data: serde_json::Value,
}

/// Best-effort conversion of a provider-supplied billing-period end into a Unix
/// timestamp in **seconds**. Providers send this in wildly different shapes:
/// Stripe/Razorpay as an epoch integer, Razorpay as an epoch-*string*,
/// Paddle/Polar/LemonSqueezy/Revolut/Airwallex as an RFC 3339 string.
///
/// Anything that fails to parse yields `None` — by design. The paywall fails
/// *closed* on a missing period end (audit F#5), so an unparseable value denies
/// rather than grants forever.
pub fn period_end_to_unix(value: Option<&serde_json::Value>) -> Option<i64> {
    let v = value?;
    if let Some(n) = v.as_i64() {
        // Heuristic: a magnitude past ~year 2286 in seconds is really
        // milliseconds (Razorpay/PayPal send ms). Normalize to seconds.
        return Some(if n > 1_000_000_000_000 { n / 1000 } else { n });
    }
    if let Some(s) = v.as_str() {
        let trimmed = s.trim();
        if let Ok(n) = trimmed.parse::<i64>() {
            return Some(if n > 1_000_000_000_000 { n / 1000 } else { n });
        }
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
            return Some(dt.timestamp());
        }
    }
    None
}

/// Common billing operations every provider must support.
#[async_trait]
pub trait BillingProvider: Send + Sync {
    /// Name of this provider (e.g., "stripe", "polar").
    fn provider_name(&self) -> &'static str;

    /// Create a checkout session for a plan.
    async fn create_checkout(
        &self,
        plan_slug: &str,
        customer_email: &str,
        user_id: i32,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError>;

    /// Create a one-time checkout for a per-post purchase. Providers that
    /// support single payments (e.g. Stripe) override this; the default returns
    /// "not supported" so per-post purchases simply aren't offered until a
    /// provider implements it. The grant of `post_purchases` happens in the
    /// verified webhook from the server-bound checkout intent — not here.
    #[allow(clippy::too_many_arguments)]
    async fn create_post_checkout(
        &self,
        _post_id: i32,
        _amount_cents: i32,
        _currency: &str,
        _customer_email: &str,
        _user_id: i32,
        _success_url: &str,
        _cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        Err(BillingError::Config(format!(
            "per-post checkout not supported by provider '{}'",
            self.provider_name()
        )))
    }

    /// Cancel a subscription at the provider.
    async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        immediately: bool,
    ) -> Result<(), BillingError>;

    /// Get subscription info from the provider.
    async fn get_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<SubscriptionInfo, BillingError>;

    /// Verify and parse an incoming webhook.
    async fn verify_webhook(&self, event: WebhookEvent) -> Result<ParsedWebhook, BillingError>;

    /// Create a billing portal session for the customer to manage their subscription.
    async fn create_portal_session(
        &self,
        provider_customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError>;
}

/// Errors from billing operations.
#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider API error: {0}")]
    ProviderApi(String),

    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::{canonical_subscription_status, period_end_to_unix};
    use serde_json::json;

    #[test]
    fn period_end_handles_epoch_seconds_int() {
        // Stripe/Razorpay send Unix seconds as a JSON integer.
        assert_eq!(
            period_end_to_unix(Some(&json!(1_700_000_000))),
            Some(1_700_000_000)
        );
    }

    #[test]
    fn period_end_normalizes_milliseconds_int() {
        // Some providers send milliseconds; the magnitude disambiguates.
        assert_eq!(
            period_end_to_unix(Some(&json!(1_700_000_000_000i64))),
            Some(1_700_000_000)
        );
    }

    #[test]
    fn period_end_handles_epoch_string() {
        // Razorpay sometimes sends the epoch as a JSON string.
        assert_eq!(
            period_end_to_unix(Some(&json!("1700000000"))),
            Some(1_700_000_000)
        );
    }

    #[test]
    fn period_end_handles_rfc3339_string() {
        // Paddle/Polar/LemonSqueezy/Revolut send RFC 3339.
        let v = json!("2023-11-14T22:13:20Z");
        assert_eq!(period_end_to_unix(Some(&v)), Some(1_700_000_000));
    }

    #[test]
    fn period_end_none_for_garbage_or_missing() {
        // Fail-closed: anything unparseable (and a missing value) yields None,
        // which the paywall treats as "deny".
        assert_eq!(period_end_to_unix(None), None);
        assert_eq!(period_end_to_unix(Some(&json!("not-a-date"))), None);
        assert_eq!(period_end_to_unix(Some(&json!(null))), None);
    }

    #[test]
    fn canonical_status_folds_active_vocabulary() {
        for raw in [
            "active",
            "ACTIVE", // case-insensitive
            "activated",
            "subscription_active",
            "incomplete_active",
            "running",
            "authorized", // Mercado Pago preapproval (audit F#11)
        ] {
            assert_eq!(
                canonical_subscription_status(Some(raw)),
                Some("active"),
                "raw={raw}"
            );
        }
    }

    #[test]
    fn canonical_status_folds_trialing_vocabulary() {
        for raw in [
            "trialing",
            "trialling",
            "trial",
            "in_trial",
            "pending_trial",
            "on_trial",
        ] {
            assert_eq!(
                canonical_subscription_status(Some(raw)),
                Some("trialing"),
                "raw={raw}"
            );
        }
    }

    #[test]
    fn canonical_status_folds_past_due_vocabulary() {
        for raw in [
            "past_due",
            "pastdue",
            "unpaid",
            "problem",
            "suspended",
            "paused",
            "on_hold",
            "incomplete",
        ] {
            assert_eq!(
                canonical_subscription_status(Some(raw)),
                Some("past_due"),
                "raw={raw}"
            );
        }
    }

    #[test]
    fn canonical_status_folds_canceled_vocabulary() {
        for raw in ["canceled", "cancelled", "subscription_cancelled", "revoked"] {
            assert_eq!(
                canonical_subscription_status(Some(raw)),
                Some("canceled"),
                "raw={raw}"
            );
        }
    }

    #[test]
    fn canonical_status_folds_expired_vocabulary() {
        // `completed` (Razorpay: full cycle run) and `ended` (Mercado Pago
        // preapproval terminated) are terminal — no further access (audit F#11).
        for raw in ["expired", "halted", "failed", "completed", "ended"] {
            assert_eq!(
                canonical_subscription_status(Some(raw)),
                Some("expired"),
                "raw={raw}"
            );
        }
    }

    #[test]
    fn canonical_status_none_for_unrecognized_or_missing() {
        // Unrecognized ⇒ None (the dispatch leaves the row's existing status
        // untouched — safe, fail-closed). Missing ⇒ None.
        assert_eq!(canonical_subscription_status(Some("authenticated")), None);
        assert_eq!(canonical_subscription_status(Some("nonsense")), None);
        assert_eq!(canonical_subscription_status(None), None);
    }
}
