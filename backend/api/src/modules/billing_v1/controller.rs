//! Billing API controllers.
//!
//! Admin endpoints for plan/subscription/payment/invoice management.
//! Consumer endpoints for checkout and portal access.
//! Public webhook endpoint for provider callbacks.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
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
use crate::AppState;

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
    Json(_payload): Json<CancelSubscriptionPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let sub = subscription::Entity::find_by_id(subscription_id)
        .one(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Subscription not found")
        })?;

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
    Extension(_user_id): Extension<i32>,
    Extension(_user_email): Extension<String>,
    Json(payload): Json<CreateCheckoutPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
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

    let _success_url = payload
        .success_url
        .unwrap_or_else(|| "/billing/success".to_string());
    let _cancel_url = payload
        .cancel_url
        .unwrap_or_else(|| "/billing/cancel".to_string());

    // TODO: Route to the configured provider based on plan metadata
    // For now return a placeholder
    Ok(Json(json!({
        "data": {
            "plan_id": plan.id,
            "plan_name": plan.name,
            "price_cents": plan.price_cents,
            "currency": plan.currency,
            "provider": "stripe",
            "message": "Provider checkout not yet configured — set STRIPE_SECRET_KEY"
        }
    })))
}

// ── Consumer: My subscriptions ────────────────────────────────────────

pub async fn my_subscriptions(
    State(state): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
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
    Extension(user_id): Extension<i32>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
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
    State(_state): State<AppState>,
    Path(provider): Path<String>,
    Json(_payload): Json<WebhookPayload>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // TODO: Route to the appropriate BillingProvider::verify_webhook()
    // and process the event (create/update subscription, record payment, etc.)
    Ok(Json(json!({
        "received": true,
        "provider": provider
    })))
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
