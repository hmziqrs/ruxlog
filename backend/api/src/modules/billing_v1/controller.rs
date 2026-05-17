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
use serde_json::json;

use crate::db::sea_models::discount_code;
use crate::db::sea_models::invoice;
use crate::db::sea_models::payment;
use crate::db::sea_models::plan;
use crate::db::sea_models::post_access;
use crate::db::sea_models::subscription;
use crate::error::codes::ErrorCode;
use crate::error::response::ErrorResponse;
use crate::services::auth::AuthSession;
use crate::AppState;

#[cfg(feature = "billing")]
use crate::services::billing::provider::{BillingProvider, ParsedWebhook, WebhookEvent};

use super::validator::*;

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
        let signature = extract_signature(&headers, &provider);

        let webhook_event = WebhookEvent {
            provider: provider.clone(),
            payload: body.to_vec(),
            signature,
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

fn extract_signature(headers: &HeaderMap, provider: &str) -> String {
    match provider {
        "stripe" => headers
            .get("Stripe-Signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string(),
        _ => headers
            .get("X-Signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string(),
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
            let user_id: i32 = event.data["data"]["object"]["metadata"]["user_id"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if user_id == 0 {
                tracing::warn!("checkout.session.completed without user_id in metadata");
                return Ok(());
            }

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

            // Default to first active plan if no plan specified
            let plan_id = plan::Entity::find()
                .filter(plan::Column::IsActive.eq(true))
                .one(&state.sea_db)
                .await
                .ok()
                .flatten()
                .map(|p| p.id)
                .unwrap_or(1);

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
                    Some(subscription_id)
                }),
                status: Set(subscription::model::SubscriptionStatus::Active),
                current_period_start: Set(Some(chrono::Utc::now().fixed_offset())),
                current_period_end: Set(None),
                cancel_at_period_end: Set(false),
                trial_ends_at: Set(None),
                metadata: Set(Some(event.data.clone())),
                ..Default::default()
            };
            active_model
                .insert(&state.sea_db)
                .await
                .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

            tracing::info!(user_id, "Subscription created from checkout");
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
                    let new_status = if event.event_type == "customer.subscription.deleted" {
                        subscription::model::SubscriptionStatus::Canceled
                    } else {
                        let stripe_status = event.data["data"]["object"]["status"]
                            .as_str()
                            .unwrap_or("active");
                        match stripe_status {
                            "active" => subscription::model::SubscriptionStatus::Active,
                            "past_due" => subscription::model::SubscriptionStatus::PastDue,
                            "canceled" => subscription::model::SubscriptionStatus::Canceled,
                            "trialing" => subscription::model::SubscriptionStatus::Trialing,
                            "expired" => subscription::model::SubscriptionStatus::Expired,
                            _ => subscription::model::SubscriptionStatus::Active,
                        }
                    };
                    active.status = Set(new_status);
                    active.updated_at = Set(chrono::Utc::now().fixed_offset());
                    active
                        .update(&state.sea_db)
                        .await
                        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

                    tracing::info!(
                        subscription_id = %provider_sub_id,
                        status = ?new_status,
                        "Subscription updated from webhook"
                    );
                }
            }
        }
        "invoice.payment_succeeded" => {
            let user_id: i32 = event.data["data"]["object"]["metadata"]["user_id"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let amount = event.data["data"]["object"]["amount_paid"]
                .as_i64()
                .unwrap_or(0) as i32;
            let currency = event.data["data"]["object"]["currency"]
                .as_str()
                .unwrap_or("usd")
                .to_string();

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
