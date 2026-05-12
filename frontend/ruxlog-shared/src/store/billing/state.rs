use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{PaginatedList, StateFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Plan ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub interval: String,
    pub trial_days: i32,
    pub features: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            slug: String::new(),
            description: None,
            price_cents: 0,
            currency: "USD".to_string(),
            interval: "month".to_string(),
            trial_days: 0,
            features: None,
            is_active: true,
            sort_order: 0,
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_else(|| Utc::now()),
            updated_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_else(|| Utc::now()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlansAddPayload {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub currency: String,
    pub interval: String,
    pub trial_days: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlansEditPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_cents: Option<i32>,
    pub currency: Option<String>,
    pub interval: Option<String>,
    pub trial_days: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

// ── Subscription ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub id: i32,
    pub user_id: i32,
    pub plan_id: i32,
    pub provider: String,
    pub provider_subscription_id: Option<String>,
    pub status: String,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Payment ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Payment {
    pub id: i32,
    pub user_id: i32,
    pub plan_id: Option<i32>,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Invoice ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invoice {
    pub id: i32,
    pub user_id: i32,
    pub subscription_id: Option<i32>,
    pub invoice_number: String,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub pdf_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Discount Code ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscountCode {
    pub id: i32,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: i32,
    pub currency: Option<String>,
    pub max_redemptions: Option<i32>,
    pub redeemed_count: i32,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub plan_id: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ── State ──

pub struct BillingState {
    pub plans_list: GlobalSignal<StateFrame<Vec<Plan>>>,
    pub plan_add: GlobalSignal<StateFrame<(), PlansAddPayload>>,
    pub plan_edit: GlobalSignal<HashMap<i32, StateFrame<(), PlansEditPayload>>>,
    pub plan_remove: GlobalSignal<HashMap<i32, StateFrame>>,
    pub plan_view: GlobalSignal<HashMap<i32, StateFrame<Plan>>>,

    pub subscriptions_list: GlobalSignal<StateFrame<Vec<Subscription>>>,
    pub payments_list: GlobalSignal<StateFrame<Vec<Payment>>>,
    pub invoices_list: GlobalSignal<StateFrame<Vec<Invoice>>>,

    pub discount_codes_list: GlobalSignal<StateFrame<Vec<DiscountCode>>>,
    pub discount_code_add: GlobalSignal<StateFrame<(), DiscountCodeAddPayload>>,
    pub discount_code_remove: GlobalSignal<HashMap<i32, StateFrame>>,
}

impl BillingState {
    pub fn new() -> Self {
        Self {
            plans_list: GlobalSignal::new(|| StateFrame::new()),
            plan_add: GlobalSignal::new(|| StateFrame::new()),
            plan_edit: GlobalSignal::new(|| HashMap::new()),
            plan_remove: GlobalSignal::new(|| HashMap::new()),
            plan_view: GlobalSignal::new(|| HashMap::new()),
            subscriptions_list: GlobalSignal::new(|| StateFrame::new()),
            payments_list: GlobalSignal::new(|| StateFrame::new()),
            invoices_list: GlobalSignal::new(|| StateFrame::new()),
            discount_codes_list: GlobalSignal::new(|| StateFrame::new()),
            discount_code_add: GlobalSignal::new(|| StateFrame::new()),
            discount_code_remove: GlobalSignal::new(|| HashMap::new()),
        }
    }
}

static BILLING_STATE: std::sync::OnceLock<BillingState> = std::sync::OnceLock::new();

pub fn use_billing() -> &'static BillingState {
    BILLING_STATE.get_or_init(|| BillingState::new())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DiscountCodeAddPayload {
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: i32,
    pub currency: Option<String>,
    pub max_redemptions: Option<i32>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub plan_id: Option<i32>,
    pub is_active: Option<bool>,
}
