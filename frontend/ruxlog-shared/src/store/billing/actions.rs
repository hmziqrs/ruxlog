use dioxus::prelude::*;

use super::state::*;
use oxcore::http;
use oxstore::StateFrame;
use std::collections::HashMap;

impl BillingState {
    // ── Plans ──

    pub async fn add_plan(&self, payload: PlansAddPayload) {
        let mut frame = StateFrame::new();
        frame.set_loading();
        *self.plan_add.write() = frame;
        let result = http::post("/billing/v1/plan/create", &payload).send().await;
        let mut frame = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    self.list_plans().await;
                } else {
                    frame.set_failed("Failed to create plan".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.plan_add.write() = frame;
    }

    pub async fn remove_plan(&self, id: i32) {
        let _ = http::post(&format!("/billing/v1/plan/delete/{}", id), &())
            .send()
            .await;
        self.list_plans().await;
    }

    pub async fn list_plans(&self) {
        let mut frame: StateFrame<Vec<Plan>> = StateFrame::new();
        frame.set_loading();
        *self.plans_list.write() = frame;
        let result = http::post("/billing/v1/plan/list", &()).send().await;
        let mut frame: StateFrame<Vec<Plan>> = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    match resp.json::<Vec<Plan>>().await {
                        Ok(data) => {
                            frame.set_success(Some(data));
                        }
                        Err(_) => {
                            frame.set_failed("Parse error".to_string());
                        }
                    }
                } else {
                    frame.set_failed("Failed to load plans".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.plans_list.write() = frame;
    }

    pub async fn view_plan(&self, id: i32) {
        let plans = self.plans_list.read();
        if let Some(data) = &plans.data {
            if let Some(plan) = data.iter().find(|p| p.id == id) {
                let mut frame: StateFrame<Plan> = StateFrame::new();
                frame.set_success(Some(plan.clone()));
                self.plan_view.write().insert(id, frame);
                return;
            }
        }
        drop(plans);
        self.list_plans().await;
        let plans = self.plans_list.read();
        if let Some(data) = &plans.data {
            if let Some(plan) = data.iter().find(|p| p.id == id) {
                let mut frame: StateFrame<Plan> = StateFrame::new();
                frame.set_success(Some(plan.clone()));
                self.plan_view.write().insert(id, frame);
            }
        }
    }

    // ── Subscriptions ──

    pub async fn list_subscriptions(&self) {
        let mut frame: StateFrame<Vec<Subscription>> = StateFrame::new();
        frame.set_loading();
        *self.subscriptions_list.write() = frame;
        let result = http::post("/billing/v1/subscription/list", &())
            .send()
            .await;
        let mut frame: StateFrame<Vec<Subscription>> = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    match resp.json::<Vec<Subscription>>().await {
                        Ok(data) => {
                            frame.set_success(Some(data));
                        }
                        Err(_) => {
                            frame.set_failed("Parse error".to_string());
                        }
                    }
                } else {
                    frame.set_failed("Failed to load subscriptions".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.subscriptions_list.write() = frame;
    }

    pub async fn cancel_subscription(&self, id: i32) {
        let _ = http::post(
            &format!("/billing/v1/subscription/cancel/{}", id),
            &serde_json::json!({ "immediately": true }),
        )
        .send()
        .await;
        self.list_subscriptions().await;
    }

    // ── Payments ──

    pub async fn list_payments(&self) {
        let mut frame: StateFrame<Vec<Payment>> = StateFrame::new();
        frame.set_loading();
        *self.payments_list.write() = frame;
        let result = http::post("/billing/v1/payment/list", &()).send().await;
        let mut frame: StateFrame<Vec<Payment>> = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    match resp.json::<Vec<Payment>>().await {
                        Ok(data) => {
                            frame.set_success(Some(data));
                        }
                        Err(_) => {
                            frame.set_failed("Parse error".to_string());
                        }
                    }
                } else {
                    frame.set_failed("Failed to load payments".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.payments_list.write() = frame;
    }

    // ── Invoices ──

    pub async fn list_invoices(&self) {
        let mut frame: StateFrame<Vec<Invoice>> = StateFrame::new();
        frame.set_loading();
        *self.invoices_list.write() = frame;
        let result = http::post("/billing/v1/invoice/list", &()).send().await;
        let mut frame: StateFrame<Vec<Invoice>> = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    match resp.json::<Vec<Invoice>>().await {
                        Ok(data) => {
                            frame.set_success(Some(data));
                        }
                        Err(_) => {
                            frame.set_failed("Parse error".to_string());
                        }
                    }
                } else {
                    frame.set_failed("Failed to load invoices".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.invoices_list.write() = frame;
    }

    // ── Discount Codes ──

    pub async fn list_discount_codes(&self) {
        let mut frame: StateFrame<Vec<DiscountCode>> = StateFrame::new();
        frame.set_loading();
        *self.discount_codes_list.write() = frame;
        let result = http::post("/billing/v1/discount/list", &()).send().await;
        let mut frame: StateFrame<Vec<DiscountCode>> = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    match resp.json::<Vec<DiscountCode>>().await {
                        Ok(data) => {
                            frame.set_success(Some(data));
                        }
                        Err(_) => {
                            frame.set_failed("Parse error".to_string());
                        }
                    }
                } else {
                    frame.set_failed("Failed to load discount codes".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.discount_codes_list.write() = frame;
    }

    pub async fn add_discount_code(&self, payload: DiscountCodeAddPayload) {
        let mut frame = StateFrame::new();
        frame.set_loading();
        *self.discount_code_add.write() = frame;
        let result = http::post("/billing/v1/discount/create", &payload)
            .send()
            .await;
        let mut frame = StateFrame::new();
        match result {
            Ok(resp) => {
                if (200..300).contains(&resp.status()) {
                    self.list_discount_codes().await;
                } else {
                    frame.set_failed("Failed to create discount code".to_string());
                }
            }
            Err(_) => {
                frame.set_failed("Network error".to_string());
            }
        }
        *self.discount_code_add.write() = frame;
    }

    pub async fn remove_discount_code(&self, id: i32) {
        let _ = http::post(&format!("/billing/v1/discount/delete/{}", id), &())
            .send()
            .await;
        self.list_discount_codes().await;
    }

    pub fn reset(&self) {
        *self.plans_list.write() = StateFrame::new();
        *self.plan_add.write() = StateFrame::new();
        *self.plan_edit.write() = HashMap::new();
        *self.plan_remove.write() = HashMap::new();
        *self.plan_view.write() = HashMap::new();
        *self.subscriptions_list.write() = StateFrame::new();
        *self.payments_list.write() = StateFrame::new();
        *self.invoices_list.write() = StateFrame::new();
        *self.discount_codes_list.write() = StateFrame::new();
        *self.discount_code_add.write() = StateFrame::new();
        *self.discount_code_remove.write() = HashMap::new();
    }
}
