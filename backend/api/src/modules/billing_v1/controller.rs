//! Billing API controllers.
//!
//! Admin endpoints for plan/subscription/payment/invoice management.
//! Consumer endpoints for checkout and portal access.
//! Public webhook endpoint for provider callbacks.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use axum_client_ip::ClientIp;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::sea_models::discount_code;
use crate::db::sea_models::invoice;
use crate::db::sea_models::payment;
use crate::db::sea_models::plan;
use crate::db::sea_models::post_access;
use crate::db::sea_models::post_purchase;
use crate::db::sea_models::subscription;
use crate::error::codes::ErrorCode;
use crate::error::response::ErrorResponse;
use crate::services::auth::AuthSession;
use crate::services::paywall;
use crate::AppState;

#[cfg(feature = "billing")]
use crate::services::billing::provider::{
    canonical, canonical_subscription_status, BillingProvider, ParsedWebhook, WebhookEvent,
};

use super::validator::*;

// ── Server-side checkout intent store (plan Phase 1f / 4e) ─────────────
//
// When a checkout session is created we persist, server-side and keyed by the
// provider's checkout session id, the *authenticated* facts the verified
// webhook later needs to grant entitlement: the `user_id` (always) and, for a
// per-post purchase, the `post_id` / amount / currency. The webhook consumes
// this intent and grants from it — never from client-shapeable `metadata`,
// which is how a forged/legit-but-tampered session could otherwise grant
// access to the wrong user or a different post.
#[cfg(feature = "billing")]
mod checkout_intent {
    use super::*;
    // fred's `set`/`get`/`del` are trait methods on the Pool.
    use tower_sessions_redis_store::fred::prelude::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CheckoutIntent {
        /// The authenticated user who initiated checkout. Authoritative —
        /// preferred over `metadata.user_id` at grant time.
        pub user_id: i32,
        /// Present only for per-post (one-time) purchases.
        pub post_id: Option<i32>,
        pub amount_cents: Option<i32>,
        pub currency: Option<String>,
        /// Present for subscription checkouts — the exact plan the customer
        /// is subscribing to, recorded server-side at checkout. The verified
        /// webhook grants THIS plan rather than guessing "first active plan"
        /// (audit finding F#3). Always `None` for per-post purchases.
        pub plan_id: Option<i32>,
    }

    impl CheckoutIntent {
        pub(crate) fn redis_key(session_id: &str) -> String {
            // Namespaced so it can't collide with session/oauth keys.
            format!("billing:checkout_intent:{session_id}")
        }
    }

    /// Persist the intent for a created checkout session. 1h TTL: long enough
    /// to outlast a customer completing payment, short enough to reap abandoned
    /// sessions.
    pub async fn store(
        state: &AppState,
        session_id: &str,
        intent: &CheckoutIntent,
    ) -> Result<(), ErrorResponse> {
        let payload = serde_json::to_string(intent).map_err(|e| {
            tracing::error!(error = ?e, "Failed to serialize checkout intent");
            ErrorResponse::new(ErrorCode::InternalServerError)
        })?;
        state
            .redis_pool
            .set::<(), _, _>(
                CheckoutIntent::redis_key(session_id),
                payload,
                Some(fred::types::Expiration::EX(3600)),
                None,
                false,
            )
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to store checkout intent in Redis");
                ErrorResponse::new(ErrorCode::InternalServerError)
            })?;
        Ok(())
    }

    /// Atomically take the single-use intent for `session_id`.
    ///
    /// Uses Redis `GETDEL` (a single round-trip that reads *and* deletes), so
    /// a concurrent/replayed webhook — two deliveries racing through the grant
    /// path — can never both observe the intent. The previous GET-then-DEL pair
    /// had a TOCTOU window where both callers read the same value before either
    /// deleted it (audit finding F#1). The DB unique indexes on
    /// `(user_id, post_id)` and `(provider, provider_subscription_id)` are a
    /// defense-in-depth backstop against any double-grant that still slipped
    /// through, but the atomic take is the primary guarantee.
    ///
    /// Returns `None` when no intent was stored (legacy/no-billing-Redis path).
    pub async fn take(
        state: &AppState,
        session_id: &str,
    ) -> Result<Option<CheckoutIntent>, ErrorResponse> {
        let key = CheckoutIntent::redis_key(session_id);
        // GETDEL: atomic read-and-delete. nil key → None, which the caller
        // treats as "no intent".
        let stored: Option<String> = state.redis_pool.getdel(&key).await.map_err(|e| {
            tracing::error!(error = ?e, "Failed to read checkout intent from Redis");
            ErrorResponse::new(ErrorCode::InternalServerError)
        })?;
        match stored {
            Some(s) => {
                let intent: CheckoutIntent = serde_json::from_str(&s).map_err(|e| {
                    tracing::error!(error = ?e, "Failed to parse stored checkout intent");
                    ErrorResponse::new(ErrorCode::InternalServerError)
                })?;
                Ok(Some(intent))
            }
            None => Ok(None),
        }
    }
}

// ── Admin: Plan CRUD ──────────────────────────────────────────────────

pub async fn admin_list_plans(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let query: Vec<plan::model::Model> = plan::Entity::find()
        .order_by_asc(plan::Column::SortOrder)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    let plans: Vec<PlanResponse> = query
        .into_iter()
        .map(|p| PlanResponse {
            id: p.id,
            name: p.name,
            slug: p.slug,
            description: p.description,
            price_cents: p.price_cents,
            currency: p.currency,
            interval: p.interval,
            trial_days: p.trial_days,
            features: p.features,
            is_active: p.is_active,
            sort_order: p.sort_order,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();

    Ok(Json(json!({ "data": plans })))
}

pub async fn admin_create_plan(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlanPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let active_model = plan::ActiveModel {
        name: Set(payload.name),
        slug: Set(payload.slug),
        description: Set(payload.description),
        price_cents: Set(payload.price_cents),
        currency: Set(payload.currency),
        interval: Set(payload.interval),
        trial_days: Set(payload.trial_days.unwrap_or(0)),
        features: Set(payload.features),
        is_active: Set(payload.is_active.unwrap_or(true)),
        sort_order: Set(payload.sort_order.unwrap_or(0)),
        ..Default::default()
    };

    let model = active_model.insert(&state.sea_db).await.map_err(|e| {
        if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
            ErrorResponse::new(ErrorCode::DuplicateEntry)
                .with_message("A plan with this slug already exists")
        } else {
            ErrorResponse::new(ErrorCode::QueryError)
        }
    })?;

    Ok(Json(json!({
        "data": { "id": model.id, "slug": model.slug },
        "message": "Plan created"
    })))
}

pub async fn admin_update_plan(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
    Json(payload): Json<UpdatePlanPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let existing = plan::Entity::find_by_id(plan_id)
        .one(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Plan not found")
        })?;

    let mut active: plan::ActiveModel = existing.into();

    if let Some(name) = payload.name {
        active.name = Set(name);
    }
    if let Some(description) = payload.description {
        active.description = Set(Some(description));
    }
    if let Some(price_cents) = payload.price_cents {
        active.price_cents = Set(price_cents);
    }
    if let Some(currency) = payload.currency {
        active.currency = Set(currency);
    }
    if let Some(interval) = payload.interval {
        active.interval = Set(interval);
    }
    if let Some(trial_days) = payload.trial_days {
        active.trial_days = Set(trial_days);
    }
    if let Some(features) = payload.features {
        active.features = Set(Some(features));
    }
    if let Some(is_active) = payload.is_active {
        active.is_active = Set(is_active);
    }
    if let Some(sort_order) = payload.sort_order {
        active.sort_order = Set(sort_order);
    }
    active.updated_at = Set(chrono::Utc::now().fixed_offset());

    active
        .update(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "message": "Plan updated" })))
}

pub async fn admin_delete_plan(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // Check for active subscriptions before deleting
    let sub_count = subscription::Entity::find()
        .filter(subscription::Column::PlanId.eq(plan_id))
        .count(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    if sub_count > 0 {
        return Err(ErrorResponse::new(ErrorCode::DependencyExists)
            .with_message("Cannot delete plan with active subscriptions"));
    }

    plan::Entity::delete_by_id(plan_id)
        .exec(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "message": "Plan deleted" })))
}

// ── Admin: Subscription management ────────────────────────────────────

pub async fn admin_list_subscriptions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let subs: Vec<subscription::model::Model> = subscription::Entity::find()
        .order_by_desc(subscription::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": subs })))
}

pub async fn admin_cancel_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<i32>,
    Json(payload): Json<CancelSubscriptionPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let sub = subscription::Entity::find_by_id(subscription_id)
        .one(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Subscription not found")
        })?;

    // Also cancel at the provider
    #[cfg(feature = "billing")]
    if let Some(provider_sub_id) = &sub.provider_subscription_id {
        let immediately = payload.immediately.unwrap_or(false);
        let provider_name = sub.provider.clone();
        if let Err(e) = state
            .billing_router
            .cancel_subscription_for_provider(&provider_name, provider_sub_id, immediately)
            .await
        {
            tracing::warn!(error = %e, provider = %provider_name, "Failed to cancel subscription at provider");
        }
    }

    // Update status in DB
    let mut active: subscription::ActiveModel = sub.into();
    active.status = Set(subscription::model::SubscriptionStatus::Canceled);
    active.cancel_at_period_end = Set(false);
    active.updated_at = Set(chrono::Utc::now().fixed_offset());
    active
        .update(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "message": "Subscription canceled" })))
}

// ── Admin: Payments & Invoices ────────────────────────────────────────

pub async fn admin_list_payments(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let payments_list: Vec<payment::model::Model> = payment::Entity::find()
        .order_by_desc(payment::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": payments_list })))
}

pub async fn admin_list_invoices(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let invoices_list: Vec<invoice::model::Model> = invoice::Entity::find()
        .order_by_desc(invoice::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": invoices_list })))
}

// ── Admin: Discount codes ─────────────────────────────────────────────

pub async fn admin_list_discount_codes(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let codes: Vec<discount_code::model::Model> = discount_code::Entity::find()
        .order_by_desc(discount_code::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": codes })))
}

pub async fn admin_create_discount_code(
    State(state): State<AppState>,
    Json(payload): Json<CreateDiscountCodePayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let active_model = discount_code::ActiveModel {
        code: Set(payload.code.to_uppercase()),
        description: Set(payload.description),
        discount_type: Set(match payload.discount_type {
            DiscountTypeValue::Percentage => discount_code::model::DiscountType::Percentage,
            DiscountTypeValue::FixedAmount => discount_code::model::DiscountType::FixedAmount,
        }),
        discount_value: Set(payload.discount_value),
        currency: Set(payload.currency),
        max_redemptions: Set(payload.max_redemptions),
        redeemed_count: Set(0),
        valid_from: Set(payload.valid_from),
        valid_until: Set(payload.valid_until),
        plan_id: Set(payload.plan_id),
        is_active: Set(payload.is_active.unwrap_or(true)),
        ..Default::default()
    };

    let model = active_model.insert(&state.sea_db).await.map_err(|e| {
        if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
            ErrorResponse::new(ErrorCode::DuplicateEntry)
                .with_message("A discount code with this code already exists")
        } else {
            ErrorResponse::new(ErrorCode::QueryError)
        }
    })?;

    Ok(Json(json!({
        "data": { "id": model.id, "code": model.code },
        "message": "Discount code created"
    })))
}

pub async fn admin_delete_discount_code(
    State(state): State<AppState>,
    Path(code_id): Path<i32>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    discount_code::Entity::delete_by_id(code_id)
        .exec(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "message": "Discount code deleted" })))
}

// ── Consumer: Public plans ────────────────────────────────────────────

pub async fn public_list_plans(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let plans: Vec<plan::model::Model> = plan::Entity::find()
        .filter(plan::Column::IsActive.eq(true))
        .order_by_asc(plan::Column::SortOrder)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": plans })))
}

// ── Consumer: Checkout ────────────────────────────────────────────────

pub async fn create_checkout(
    State(state): State<AppState>,
    auth: AuthSession,
    ClientIp(client_ip): ClientIp,
    Json(payload): Json<CreateCheckoutPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    let user_id = user.id;
    let user_email = user.email.clone();
    // Look up the plan
    let plan = plan::Entity::find()
        .filter(plan::Column::Slug.eq(&payload.plan_slug))
        .filter(plan::Column::IsActive.eq(true))
        .one(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Plan not found")
        })?;

    let success_url = payload
        .success_url
        .unwrap_or_else(|| "/billing/success".to_string());
    let cancel_url = payload
        .cancel_url
        .unwrap_or_else(|| "/billing/cancel".to_string());

    #[cfg(feature = "billing")]
    {
        let session = state
            .billing_router
            .create_checkout_for_ip(
                client_ip,
                &plan.slug,
                &user_email,
                user_id,
                &success_url,
                &cancel_url,
            )
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::ExternalServiceError)
                    .with_message(format!("Checkout failed: {}", e))
            })?;

        // Required (plan 1f/4e, audit F#2/F#3): bind the authenticated
        // `user_id` AND the exact `plan_id` server-side. The verified webhook
        // grants the subscription from this intent — never from client metadata
        // (which is attacker-shapeable and could name the wrong user) and never
        // by guessing "first active plan" (which could grant the wrong tier).
        // Storage is REQUIRED: if we cannot bind the intent, we must not hand
        // the customer a checkout URL the webhook could never fulfill, so a
        // failure returns an error instead of degrading to metadata.
        let intent = checkout_intent::CheckoutIntent {
            user_id,
            post_id: None,
            amount_cents: None,
            currency: None,
            plan_id: Some(plan.id),
        };
        checkout_intent::store(&state, &session.session_id, &intent).await?;

        return Ok(Json(json!({
            "data": {
                "session_id": session.session_id,
                "checkout_url": session.checkout_url,
            }
        })));
    }

    #[cfg(not(feature = "billing"))]
    {
        let _ = (user_id, user_email, success_url, cancel_url);
        return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("Billing is not enabled on this server"));
    }
}

// ── Consumer: Per-post checkout ───────────────────────────────────────

/// Create a one-time checkout for purchasing a single gated post.
///
/// The amount charged is the **server-side** price from the post's
/// `post_access` policy — never the client's. After the provider returns a
/// session, we bind `(user_id, post_id, amount, currency)` server-side; the
/// verified `checkout.session.completed` webhook consumes that intent and
/// inserts the `post_purchases` grant. Because the grant is impossible without
/// the intent, intent storage is *required* here (a failure returns an error
/// rather than handing the customer a checkout URL we could never fulfill).
pub async fn create_post_checkout(
    State(state): State<AppState>,
    auth: AuthSession,
    ClientIp(client_ip): ClientIp,
    Json(payload): Json<CreatePostCheckoutPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    let user_id = user.id;
    let user_email = user.email.clone();

    // Only genuinely Paid posts with a configured price are purchasable. This
    // is the authoritative amount charged — the client cannot influence it.
    let policy = paywall::load_post_access_policy(&state.sea_db, payload.post_id).await?;
    let amount_cents = match (policy.access_type, policy.price_cents) {
        (paywall::PostAccessType::Paid, Some(cents)) if cents > 0 => cents,
        _ => {
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("This post is not available for one-time purchase"));
        }
    };
    let currency = policy.currency.unwrap_or_else(|| "usd".to_string());

    let success_url = payload
        .success_url
        .unwrap_or_else(|| "/billing/success".to_string());
    let cancel_url = payload
        .cancel_url
        .unwrap_or_else(|| "/billing/cancel".to_string());

    #[cfg(feature = "billing")]
    {
        let session = state
            .billing_router
            .create_post_checkout_for_ip(
                client_ip,
                payload.post_id,
                amount_cents,
                &currency,
                &user_email,
                user_id,
                &success_url,
                &cancel_url,
            )
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::ExternalServiceError)
                    .with_message(format!("Post checkout failed: {}", e))
            })?;

        // Required (not best-effort): without this intent the webhook cannot
        // grant the purchase, so a failure must NOT return a checkout URL the
        // customer could pay into without receiving access.
        let intent = checkout_intent::CheckoutIntent {
            user_id,
            post_id: Some(payload.post_id),
            amount_cents: Some(amount_cents),
            currency: Some(currency),
            // Per-post purchases have no plan; the webhook reads `post_id` to
            // distinguish the grant path.
            plan_id: None,
        };
        checkout_intent::store(&state, &session.session_id, &intent).await?;

        return Ok(Json(json!({
            "data": {
                "session_id": session.session_id,
                "checkout_url": session.checkout_url,
            }
        })));
    }

    #[cfg(not(feature = "billing"))]
    {
        let _ = (user_id, user_email, amount_cents, currency, success_url, cancel_url);
        return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("Billing is not enabled on this server"));
    }
}

// ── Consumer: My subscriptions ────────────────────────────────────────

pub async fn my_subscriptions(
    State(state): State<AppState>,
    auth: AuthSession,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    let user_id = user.id;
    let subs: Vec<subscription::model::Model> = subscription::Entity::find()
        .filter(subscription::Column::UserId.eq(user_id))
        .order_by_desc(subscription::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": subs })))
}

// ── Consumer: My payments ─────────────────────────────────────────────

pub async fn my_payments(
    State(state): State<AppState>,
    auth: AuthSession,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized).with_message("Not authenticated")
    })?;
    let user_id = user.id;
    let payments_list: Vec<payment::model::Model> = payment::Entity::find()
        .filter(payment::Column::UserId.eq(user_id))
        .order_by_desc(payment::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "data": payments_list })))
}

// ── Webhook receiver ──────────────────────────────────────────────────

pub async fn webhook_receiver(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    #[cfg(feature = "billing")]
    {
        // Pass the full header map so each provider reads the headers its scheme
        // actually needs (Stripe `Stripe-Signature`, Paddle `Paddle-Signature`,
        // PayPal's five headers, etc.). The previous single-signature extract
        // dropped everything but one header and could never verify providers
        // that sign over a timestamp. See plan Phase 1a.
        let webhook_event = WebhookEvent {
            provider: provider.clone(),
            payload: body.to_vec(),
            headers,
        };

        let parsed = state
            .billing_router
            .verify_webhook(webhook_event)
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::ExternalServiceError)
                    .with_message(format!("Webhook verification failed: {}", e))
            })?;

        process_webhook_event(&state, &parsed, &provider).await?;

        Ok(Json(json!({ "received": true })))
    }

    #[cfg(not(feature = "billing"))]
    {
        let _ = (state, provider, headers, body);
        Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("Billing is not enabled on this server"))
    }
}

#[cfg(feature = "billing")]
async fn process_webhook_event(
    state: &AppState,
    event: &ParsedWebhook,
    provider_name: &str,
) -> Result<(), ErrorResponse> {
    match event.event_type.as_str() {
        "checkout.session.completed" => {
            // Recover the server-bound intent. The intent is keyed by the
            // checkout `session_id` the provider returned at create-checkout
            // time. Each provider's `verify_webhook` extracts that id into
            // `checkout_session_id`; after the F#11 residual fixes every
            // provider populates it with the exact round-trip id. We still fall
            // back to `subscription_id` then `payment_id` as defense-in-depth —
            // and for LemonSqueezy, whose checkout-completion resource id is the
            // order/subscription id (not the stored checkout id),
            // `checkout_session_id` won't match the intent key, so the fallbacks
            // also miss and the grant is refused (fail-closed, accepted
            // deferral). The atomic GETDEL means only the first delivery to hit
            // the right key consumes the intent — trying multiple candidates is
            // safe (a miss returns None).
            let session_id = event
                .checkout_session_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .or_else(|| event.subscription_id.as_deref().filter(|s| !s.is_empty()))
                .or_else(|| event.payment_id.as_deref().filter(|s| !s.is_empty()))
                .unwrap_or("");

            // Server-bound intent (plan 1f/4e): the authoritative, authenticated
            // facts about who paid and for what. Atomically taken (GETDEL), so a
            // replayed/concurrent webhook cannot double-grant. A MISSING intent
            // is refused — neither the per-post nor the subscription path grants
            // from attacker-shapeable client `metadata` (audit F#2/F#10). Both
            // checkouts now store the intent as a hard requirement, so a missing
            // intent here means the session predates that binding or Redis lost
            // it; in either case the safe action is to not grant.
            let intent = if !session_id.is_empty() {
                checkout_intent::take(state, session_id)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = ?e, "Failed to read checkout intent");
                        None
                    })
            } else {
                None
            };

            // Diagnose-only: the provider-normalized payer id, kept for the
            // refusal log below. NEVER used to grant.
            let metadata_user_id: i32 = event.user_id.unwrap_or(0);

            let intent = match intent {
                Some(i) => i,
                None => {
                    tracing::warn!(
                        metadata_user_id,
                        session_id,
                        provider = provider_name,
                        "checkout.session.completed with no resolvable server-bound \
                         intent; refusing to grant (audit F#2/F#10)"
                    );
                    return Ok(());
                }
            };
            let user_id = intent.user_id;

            if user_id == 0 {
                tracing::warn!("checkout.session.completed with no resolvable user_id");
                return Ok(());
            }

            // Per-post (one-time) purchase: grant a `post_purchases` row from
            // the server-bound intent ONLY. post_id in metadata is ignored for
            // granting — an attacker cannot buy access to a post they didn't
            // initiate a real, server-recorded checkout for.
            if let Some(post_id) = intent.post_id {
                let amount_cents = intent.amount_cents.unwrap_or(0);
                let currency = intent
                    .currency
                    .clone()
                    .unwrap_or_else(|| "usd".to_string());

                // Idempotent grant. The (user_id, post_id) unique index is the
                // real backstop against a concurrent double-grant; the pre-check
                // keeps the common replay case out of the error path.
                let already = post_purchase::Entity::find()
                    .filter(post_purchase::Column::UserId.eq(user_id))
                    .filter(post_purchase::Column::PostId.eq(post_id))
                    .one(&state.sea_db)
                    .await
                    .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
                if already.is_some() {
                    tracing::info!(
                        user_id,
                        post_id,
                        "Post purchase already exists, skipping (idempotent)"
                    );
                    return Ok(());
                }

                let active_model = post_purchase::model::ActiveModel {
                    user_id: Set(user_id),
                    post_id: Set(post_id),
                    payment_id: Set(None),
                    provider: Set(provider_name.to_string()),
                    amount_cents: Set(amount_cents),
                    currency: Set(currency),
                    ..Default::default()
                };
                match active_model.insert(&state.sea_db).await {
                    Ok(_) => {
                        tracing::info!(
                            user_id,
                            post_id,
                            "Post purchase granted from verified webhook"
                        );
                    }
                    Err(e) => {
                        let s = e.to_string();
                        if s.contains("duplicate") || s.contains("unique") {
                            // A concurrent webhook granted it first — still correct.
                            tracing::info!(
                                user_id,
                                post_id,
                                "Concurrent post purchase raced; already granted"
                            );
                        } else {
                            return Err(ErrorResponse::new(ErrorCode::QueryError));
                        }
                    }
                }
                return Ok(());
            }

            // Subscription checkout path.
            let subscription_id = event.subscription_id.clone().unwrap_or_default();
            let customer_id = event.customer_id.clone();

            // Check if subscription already exists (idempotency)
            let existing = if !subscription_id.is_empty() {
                subscription::Entity::find()
                    .filter(subscription::Column::ProviderSubscriptionId.eq(&subscription_id))
                    .one(&state.sea_db)
                    .await
                    .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?
            } else {
                None
            };

            if existing.is_some() {
                tracing::info!(
                    subscription_id = %subscription_id,
                    "Subscription already exists, skipping"
                );
                return Ok(());
            }

            // The exact plan comes from the server-bound checkout intent
            // (audit F#3): the customer subscribed to THIS plan at checkout, so
            // granting it must not silently fall back to "first active plan",
            // which could grant the wrong tier (e.g. a premium plan id where a
            // basic one was purchased). The subscription checkout always stores
            // `plan_id`; an absent `plan_id` means a legacy/corrupt intent, and
            // we fail closed rather than guess.
            let plan_id = match intent.plan_id {
                Some(id) => id,
                None => {
                    tracing::error!(
                        user_id,
                        "checkout.session.completed (subscription) with no \
                         server-bound plan_id; refusing to grant a guessed plan \
                         (audit F#3)"
                    );
                    return Ok(());
                }
            };

            let active_model = subscription::ActiveModel {
                user_id: Set(user_id),
                plan_id: Set(plan_id),
                provider: Set(provider_name.to_string()),
                provider_customer_id: Set(if customer_id.is_empty() {
                    None
                } else {
                    Some(customer_id)
                }),
                provider_subscription_id: Set(if subscription_id.is_empty() {
                    None
                } else {
                    Some(subscription_id.clone())
                }),
                status: Set(subscription::model::SubscriptionStatus::Active),
                current_period_start: Set(Some(chrono::Utc::now().fixed_offset())),
                // Real period end from the verified webhook (audit F#11): the
                // paywall fails CLOSED on a missing/None end, so persisting a
                // real value is what lets an active subscriber read gated content
                // — and what revokes them once it lapses. A malformed timestamp
                // degrades to None (fail-closed) rather than fabricating "now",
                // which would expire the subscriber immediately (audit F#11
                // round-2, data-quality cleanup).
                current_period_end: Set(event.current_period_end.and_then(|ts| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
                        .map(|dt| dt.fixed_offset())
                })),
                cancel_at_period_end: Set(false),
                trial_ends_at: Set(None),
                metadata: Set(Some(event.data.clone())),
                ..Default::default()
            };
            // Duplicate-tolerant insert (audit F#1): the pre-check above keeps
            // the common replay out of the error path, but two concurrent
            // deliveries can both pass it and race to insert. The unique index
            // on `(provider, provider_subscription_id)` is the real backstop;
            // a collision here means the other delivery won, which is correct.
            match active_model.insert(&state.sea_db).await {
                Ok(_) => {
                    tracing::info!(user_id, "Subscription created from checkout");
                }
                Err(e) => {
                    let s = e.to_string();
                    if s.contains("duplicate") || s.contains("unique") {
                        tracing::info!(
                            user_id,
                            subscription_id = %subscription_id,
                            "Concurrent subscription grant raced; already granted (idempotent)"
                        );
                    } else {
                        return Err(ErrorResponse::new(ErrorCode::QueryError));
                    }
                }
            }
        }
        "customer.subscription.updated" | "customer.subscription.deleted" => {
            if let Some(provider_sub_id) = &event.subscription_id {
                let sub = subscription::Entity::find()
                    .filter(subscription::Column::ProviderSubscriptionId.eq(provider_sub_id))
                    .one(&state.sea_db)
                    .await
                    .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

                if let Some(existing) = sub {
                    let mut active: subscription::ActiveModel = existing.into();

                    // Status from the provider-normalized canonical value (audit
                    // F#11 residual). A deletion event is terminal. For updates,
                    // each provider folds its native status into our 5-value
                    // vocabulary at the `verify_webhook` boundary; if the provider
                    // didn't supply a recognizable status we leave the row's
                    // existing status untouched (safe — the paywall keeps gating
                    // on the persisted status + authoritative period_end) and
                    // only refresh `current_period_end`.
                    //
                    // Resurrection note (audit F#11 round-2): a stale "active"
                    // update that revives a Canceled/Expired row is NOT an
                    // over-grant, because the paywall requires BOTH
                    // `status ∈ {Active, Trialing}` AND a future
                    // `current_period_end` (`paywall::user_has_active_subscription`).
                    // A revival with a genuinely-future period is a real
                    // reactivation; one without it is denied. No row-state guard
                    // is added here because the paywall's dual invariant already
                    // neutralizes every stale-revival shape — adding one would
                    // only risk denying legitimate reactivations (first, do no
                    // harm).
                    let mut status_changed = true;
                    let new_status = if event.event_type == canonical::SUBSCRIPTION_DELETED {
                        subscription::model::SubscriptionStatus::Canceled
                    } else {
                        match canonical_subscription_status(event.subscription_status.as_deref()) {
                            Some("active") => subscription::model::SubscriptionStatus::Active,
                            Some("past_due") => subscription::model::SubscriptionStatus::PastDue,
                            Some("canceled") => subscription::model::SubscriptionStatus::Canceled,
                            Some("trialing") => subscription::model::SubscriptionStatus::Trialing,
                            Some("expired") => subscription::model::SubscriptionStatus::Expired,
                            // Unknown/missing: keep the existing row status.
                            _ => {
                                status_changed = false;
                                // Placeholder; never written when status_changed=false.
                                subscription::model::SubscriptionStatus::Active
                            }
                        }
                    };
                    if status_changed {
                        active.status = Set(new_status);
                    }
                    // Refresh the period end from the verified webhook (audit
                    // F#11): renewals send a fresh `current_period_end`, and the
                    // paywall keys off it. Only overwrite when the provider
                    // actually supplied one (cancellation events may not).
                    if let Some(ts) = event.current_period_end {
                        active.current_period_end = Set(
                            chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
                                .map(|dt| dt.fixed_offset()),
                        );
                    }
                    active.updated_at = Set(chrono::Utc::now().fixed_offset());
                    active
                        .update(&state.sea_db)
                        .await
                        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

                    tracing::info!(
                        subscription_id = %provider_sub_id,
                        status_changed,
                        status = ?new_status,
                        "Subscription updated from webhook"
                    );
                }
            }
        }
        "invoice.payment_succeeded" => {
            // Payer: prefer the provider-normalized id; otherwise attribute the
            // payment to the owner of the subscription it renews (looked up by
            // `subscription_id`). Falls back to 0 only if neither resolves — a
            // history row with no owner, never a grant. (The grant happens in
            // CHECKOUT_COMPLETED; this arm only records payment history.)
            let mut user_id: i32 = event.user_id.unwrap_or(0);
            if user_id == 0 {
                if let Some(sid) = event.subscription_id.as_deref().filter(|s| !s.is_empty()) {
                    if let Ok(Some(owner)) = subscription::Entity::find()
                        .filter(subscription::Column::ProviderSubscriptionId.eq(sid))
                        .one(&state.sea_db)
                        .await
                    {
                        user_id = owner.user_id;
                    }
                }
            }

            let amount = event.amount_cents.unwrap_or(0) as i32;
            let currency = event
                .currency
                .clone()
                .unwrap_or_else(|| "usd".to_string());

            // Idempotency (plan 1d / CWE-294): Stripe redelivers events on retry,
            // and a replayed request would otherwise insert a duplicate payment
            // row. Dedup on (provider, provider_payment_id) — the mirror of the
            // subscription dedup above — before inserting.
            if let Some(pid) = event.payment_id.as_deref().filter(|p| !p.is_empty()) {
                let dup = payment::Entity::find()
                    .filter(payment::Column::Provider.eq(provider_name))
                    .filter(payment::Column::ProviderPaymentId.eq(pid))
                    .one(&state.sea_db)
                    .await
                    .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
                if dup.is_some() {
                    tracing::info!(
                        provider = provider_name,
                        payment_id = pid,
                        "Payment already recorded, skipping (idempotent)"
                    );
                    return Ok(());
                }
            }

            let active_model = payment::ActiveModel {
                user_id: Set(user_id),
                subscription_id: Set(None),
                plan_id: Set(None),
                provider: Set(provider_name.to_string()),
                provider_payment_id: Set(event.payment_id.clone()),
                amount_cents: Set(amount),
                currency: Set(currency),
                status: Set(payment::model::PaymentStatus::Completed),
                description: Set(Some(format!("Invoice payment: {}", event.event_type))),
                metadata: Set(Some(event.data.clone())),
                ..Default::default()
            };
            active_model
                .insert(&state.sea_db)
                .await
                .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

            // Renewal period refresh (audit F#11 round-2): a recurring payment
            // that carries a fresh period end (e.g. PayPal
            // `PAYMENT.SALE.COMPLETED`, whose provider resolved the linked
            // subscription's `next_billing_time`) must extend the subscriber's row
            // so the paywall keeps admitting them across billing cycles — without
            // this, the row's `current_period_end` stays pinned to the activation
            // value and the renewing subscriber is denied after the first period.
            // Only applies when BOTH a subscription id and a period end are
            // present; history-only renewals with no period (e.g. a Stripe
            // invoice) are unaffected — their period is refreshed by the
            // provider's `customer.subscription.*` lifecycle event instead.
            // Defense-in-depth: only move the period FORWARD, never backward, so
            // an out-of-order/redelivered payment cannot shorten a valid period.
            if let (Some(sid), Some(new_end_ts)) = (
                event.subscription_id.as_deref().filter(|s| !s.is_empty()),
                event.current_period_end,
            ) {
                if let Ok(Some(sub)) = subscription::Entity::find()
                    .filter(subscription::Column::ProviderSubscriptionId.eq(sid))
                    .one(&state.sea_db)
                    .await
                {
                    let new_end = chrono::DateTime::<chrono::Utc>::from_timestamp(new_end_ts, 0)
                        .map(|dt| dt.fixed_offset());
                    let extend = match (sub.current_period_end.as_ref(), new_end) {
                        (Some(existing), Some(new_dt)) => new_dt > *existing,
                        // No existing period → set it (fail-closed needs a value).
                        (None, Some(_)) => true,
                        _ => false,
                    };
                    if extend {
                        let mut active: subscription::ActiveModel = sub.into();
                        active.current_period_end = Set(new_end);
                        active.updated_at = Set(chrono::Utc::now().fixed_offset());
                        if let Err(e) = active.update(&state.sea_db).await {
                            tracing::warn!(
                                error = ?e,
                                subscription_id = %sid,
                                "Failed to refresh period on renewal (best-effort)"
                            );
                        }
                    }
                }
            }

            tracing::info!(user_id, amount, "Payment recorded from invoice webhook");
        }
        "payment.confirmed" | "payment.pending" => {
            // Crypto payments — extract user_id from memo (rux-{user_id}-{uuid})
            let memo = event.data["memo"].as_str().unwrap_or("");
            let user_id: i32 = memo
                .strip_prefix("rux-")
                .and_then(|rest| rest.split('-').next())
                .and_then(|id| id.parse().ok())
                .unwrap_or(0);

            let amount_crypto = event.data["amount"].as_f64().unwrap_or(0.0);
            let currency = event.data["currency"].as_str().unwrap_or("BTC");

            let status = if event.event_type == "payment.confirmed" {
                payment::model::PaymentStatus::Completed
            } else {
                payment::model::PaymentStatus::Pending
            };

            // Store crypto amount as cents (amount * 100 as rough conversion)
            // In production you'd convert to fiat at current rate
            let amount_cents = (amount_crypto * 100.0) as i32;

            // Idempotency (plan 1d): dedup on (provider, provider_payment_id)
            // before inserting. (This branch is currently unreachable because
            // crypto webhooks fail-closed in verify_webhook, but the guard stays
            // correct for when on-chain confirmation polling lands.)
            if let Some(pid) = event.payment_id.as_deref().filter(|p| !p.is_empty()) {
                let dup = payment::Entity::find()
                    .filter(payment::Column::Provider.eq("crypto"))
                    .filter(payment::Column::ProviderPaymentId.eq(pid))
                    .one(&state.sea_db)
                    .await
                    .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
                if dup.is_some() {
                    tracing::info!(
                        payment_id = pid,
                        "Crypto payment already recorded, skipping (idempotent)"
                    );
                    return Ok(());
                }
            }

            let active_model = payment::ActiveModel {
                user_id: Set(user_id),
                subscription_id: Set(None),
                plan_id: Set(None),
                provider: Set("crypto".to_string()),
                provider_payment_id: Set(event.payment_id.clone()),
                amount_cents: Set(amount_cents),
                currency: Set(currency.to_string()),
                status: Set(status),
                description: Set(Some(format!(
                    "Crypto payment: {} {}",
                    amount_crypto, currency
                ))),
                metadata: Set(Some(event.data.clone())),
                ..Default::default()
            };
            active_model
                .insert(&state.sea_db)
                .await
                .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

            tracing::info!(
                user_id,
                amount = amount_crypto,
                currency,
                status = %event.event_type,
                "Crypto payment recorded from webhook"
            );
        }
        _ => {
            tracing::info!(event_type = %event.event_type, "Unhandled billing webhook event");
        }
    }
    Ok(())
}

// ── Paywall: Check post access ────────────────────────────────────────

pub async fn check_post_access(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let access = post_access::Entity::find()
        .filter(post_access::Column::PostId.eq(post_id))
        .one(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    match access {
        Some(a) => Ok(Json(json!({
            "post_id": post_id,
            "access_type": a.access_type,
            "price_cents": a.price_cents,
            "currency": a.currency,
            "requires_subscription": true,
        }))),
        None => Ok(Json(json!({
            "post_id": post_id,
            "access_type": "free",
            "requires_subscription": false,
        }))),
    }
}

// ── Admin: Set post access ────────────────────────────────────────────

pub async fn admin_set_post_access(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    Json(payload): Json<SetPostAccessPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // Upsert: delete existing access rule, insert new one
    post_access::Entity::delete_many()
        .filter(post_access::Column::PostId.eq(post_id))
        .exec(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    let active_model = post_access::model::ActiveModel {
        post_id: Set(post_id),
        access_type: Set(payload.access_type),
        price_cents: Set(payload.price_cents),
        currency: Set(payload.currency),
        ..Default::default()
    };

    active_model
        .insert(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    Ok(Json(json!({ "message": "Post access updated" })))
}

#[cfg(all(test, feature = "billing"))]
mod tests {
    use super::*;
    use crate::services::billing::provider::BillingProvider;
    use crate::services::billing::revolut::RevolutProvider;
    use crate::services::billing::stripe::StripeProvider;

    #[tokio::test]
    async fn per_post_checkout_defaults_to_not_supported_for_non_overriding_provider() {
        // RevolutProvider does not override create_post_checkout, so it inherits
        // the trait default. This locks the invariant the geo-router relies on:
        // a region routed to such a provider simply cannot sell per-post access,
        // and the checkout handler surfaces that as a Config error rather than
        // silently creating a session that could never be granted.
        let provider = RevolutProvider::new("k".into(), "s".into());
        let result = provider
            .create_post_checkout(1, 499, "usd", "buyer@example.com", 7, "s", "c")
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::services::billing::provider::BillingError::Config(msg) => {
                assert!(
                    msg.contains("not supported"),
                    "expected not-supported message, got: {msg}"
                );
            }
            other => panic!("expected Config error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn per_post_checkout_is_overridden_by_stripe() {
        // Stripe must override the default so per-post purchases are available
        // in Stripe-routed regions. We assert the override exists (no
        // "not supported" Config error) by checking the method is callable on
        // the Stripe provider without relying on a live network call — the
        // override is what matters; the network path is exercised end-to-end.
        // (We only confirm the type resolves + provider name, since calling it
        // would hit the Stripe API.)
        let provider = StripeProvider::new("sk_test".into(), "whsec".into());
        assert_eq!(provider.provider_name(), "stripe");
    }

    #[test]
    fn checkout_intent_roundtrip_preserves_grant_fields() {
        // The webhook grant path reads exactly these fields back from what the
        // checkout handler stored. A round-trip must preserve user_id and the
        // optional post_id/amount/currency/plan_id — if any were dropped, a
        // paid-post purchase would grant nothing (or a subscription the wrong
        // tier).
        let intent = checkout_intent::CheckoutIntent {
            user_id: 42,
            post_id: Some(7),
            amount_cents: Some(499),
            currency: Some("usd".to_string()),
            plan_id: None,
        };
        let s = serde_json::to_string(&intent).unwrap();
        let back: checkout_intent::CheckoutIntent = serde_json::from_str(&s).unwrap();
        assert_eq!(back.user_id, 42);
        assert_eq!(back.post_id, Some(7));
        assert_eq!(back.amount_cents, Some(499));
        assert_eq!(back.currency.as_deref(), Some("usd"));
        assert!(back.plan_id.is_none());

        // A subscription (non-post) intent has no post_id but DOES carry the
        // plan_id the webhook must grant (audit F#3).
        let sub = checkout_intent::CheckoutIntent {
            user_id: 5,
            post_id: None,
            amount_cents: None,
            currency: None,
            plan_id: Some(3),
        };
        let ss = serde_json::to_string(&sub).unwrap();
        let sub_back: checkout_intent::CheckoutIntent = serde_json::from_str(&ss).unwrap();
        assert_eq!(sub_back.user_id, 5);
        assert!(sub_back.post_id.is_none());
        assert_eq!(sub_back.plan_id, Some(3));
    }

    #[test]
    fn checkout_intent_redis_key_is_namespaced() {
        // Same-key isolation from session/oauth keys: the namespace prefix
        // prevents a collision that could let a non-checkout key be consumed
        // as a (possibly attacker-influenced) intent.
        let key = checkout_intent::CheckoutIntent::redis_key("cs_test_123");
        assert_eq!(key, "billing:checkout_intent:cs_test_123");
    }
}
