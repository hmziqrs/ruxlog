//! Billing API request/response types and validation.

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::plan::model::PlanInterval;

// --- Plan CRUD payloads ---

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreatePlanPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    pub description: Option<String>,
    #[validate(range(min = 0))]
    pub price_cents: i32,
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
    pub interval: PlanInterval,
    #[validate(range(min = 0))]
    pub trial_days: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    #[validate(range(min = 0))]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdatePlanPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 0))]
    pub price_cents: Option<i32>,
    pub currency: Option<String>,
    pub interval: Option<PlanInterval>,
    pub trial_days: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlanListQuery {
    pub include_inactive: Option<bool>,
}

// --- Checkout ---

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateCheckoutPayload {
    #[validate(length(min = 1, max = 255))]
    pub plan_slug: String,
    /// Override success URL (optional, server provides default)
    pub success_url: Option<String>,
    /// Override cancel URL (optional, server provides default)
    pub cancel_url: Option<String>,
}

// --- Subscription management ---

#[derive(Debug, Deserialize, Serialize)]
pub struct CancelSubscriptionPayload {
    pub immediately: Option<bool>,
}

// --- Discount codes ---

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateDiscountCodePayload {
    #[validate(length(min = 1, max = 50))]
    pub code: String,
    pub description: Option<String>,
    pub discount_type: DiscountTypeValue,
    #[validate(range(min = 0))]
    pub discount_value: i32,
    pub currency: Option<String>,
    pub max_redemptions: Option<i32>,
    pub valid_from: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub valid_until: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub plan_id: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscountTypeValue {
    Percentage,
    FixedAmount,
}

// --- Webhook ---

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookPayload {
    pub provider: String,
    pub payload: serde_json::Value,
    pub signature: String,
}

// --- Responses ---

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub interval: PlanInterval,
    pub trial_days: i32,
    pub features: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: i32,
    pub user_id: i32,
    pub plan_id: i32,
    pub plan_name: String,
    pub provider: String,
    pub status: String,
    pub current_period_start: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub current_period_end: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub cancel_at_period_end: bool,
    pub trial_ends_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub session_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Serialize)]
pub struct PortalResponse {
    pub portal_url: String,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: i32,
    pub user_id: i32,
    pub plan_name: Option<String>,
    pub provider: String,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: i32,
    pub invoice_number: String,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub due_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub paid_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub pdf_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
