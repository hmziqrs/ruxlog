use dioxus::prelude::*;
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::store::{use_billing, PlansEditPayload};

use crate::router::Route;

#[component]
pub fn BillingPlanEditScreen(id: i32) -> Element {
    let nav = use_navigator();
    let billing = use_billing();

    let mut name = use_signal(String::new);
    let mut slug = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut price_cents = use_signal(String::new);
    let mut currency = use_signal(|| "USD".to_string());
    let mut interval = use_signal(|| "month".to_string());
    let mut trial_days = use_signal(String::new);
    let mut sort_order = use_signal(String::new);
    let mut is_active = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut loaded = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    // Fetch plan data on mount
    use_effect(move || {
        let billing = billing;
        let plan_id = id;
        spawn(async move {
            billing.view_plan(plan_id).await;
        });
    });

    // Populate form once plan data is available
    let plan_view = billing.plan_view.read();
    if let Some(frame) = plan_view.get(&id) {
        if let Some(plan) = &frame.data {
            if !loaded() {
                name.set(plan.name.clone());
                slug.set(plan.slug.clone());
                description.set(plan.description.clone().unwrap_or_default());
                price_cents.set(plan.price_cents.to_string());
                currency.set(plan.currency.clone());
                interval.set(plan.interval.clone());
                trial_days.set(plan.trial_days.to_string());
                sort_order.set(plan.sort_order.to_string());
                is_active.set(plan.is_active);
                loaded.set(true);
            }
        }
    }
    drop(plan_view);

    let on_name_input = move |e: Event<FormData>| {
        name.set(e.value());
    };

    let on_submit = move |_| {
        let name_val = name.read().clone();
        let desc_val = description.read().clone();
        let price_val: i32 = price_cents.read().parse().unwrap_or(0);
        let curr_val = currency.read().clone();
        let int_val = interval.read().clone();
        let trial_val = trial_days.read().parse().ok();
        let sort_val = sort_order.read().parse().ok();

        if name_val.trim().is_empty() || price_val <= 0 {
            return;
        }

        let payload = PlansEditPayload {
            name: Some(name_val),
            description: if desc_val.trim().is_empty() {
                None
            } else {
                Some(desc_val)
            },
            price_cents: Some(price_val),
            currency: Some(curr_val),
            interval: Some(int_val),
            trial_days: trial_val,
            features: None,
            is_active: Some(is_active()),
            sort_order: sort_val,
        };

        let plan_id = id;
        spawn(async move {
            saving.set(true);
            let result = oxcore::http::put(
                &format!("/billing/v1/plan/update/{}", plan_id),
                &payload,
            )
            .send()
            .await;

            match result {
                Ok(resp) if (200..300).contains(&resp.status()) => {
                    let billing = billing;
                    billing.list_plans().await;
                    nav.push(Route::BillingPlansListScreen {});
                }
                Ok(resp) => {
                    error_msg.set(format!(
                        "Failed to update plan (status {})",
                        resp.status()
                    ));
                }
                Err(_) => {
                    error_msg.set("Network error".to_string());
                }
            }
            saving.set(false);
        });
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Edit Plan" }
                    p { class: "text-sm text-muted-foreground", "Update subscription plan details." }
                }
            }

            if !error_msg.read().is_empty() {
                div { class: "rounded-lg border border-red-200 bg-red-50 dark:bg-red-900/10 dark:border-red-800 p-4",
                    p { class: "text-sm text-red-800 dark:text-red-400", "{error_msg}" }
                }
            }

            if !loaded() {
                div { class: "flex items-center justify-center py-20",
                    div { class: "animate-pulse text-muted-foreground", "Loading plan..." }
                }
            } else {
                form {
                    class: "max-w-2xl space-y-6 bg-card border border-border rounded-lg p-6",
                    onsubmit: on_submit,
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Name *" }
                            input {
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                placeholder: "Pro Plan",
                                value: "{name}",
                                oninput: on_name_input,
                                required: true,
                            }
                        }
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium text-muted-foreground", "Slug" }
                            input {
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm font-mono bg-muted/50",
                                disabled: true,
                                value: "{slug}",
                            }
                        }
                    }
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Description" }
                        textarea {
                            class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm min-h-[80px] resize-y",
                            placeholder: "Plan description...",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                        }
                    }
                    div { class: "grid grid-cols-3 gap-4",
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Price (cents) *" }
                            input {
                                r#type: "number",
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                placeholder: "999",
                                value: "{price_cents}",
                                oninput: move |e| price_cents.set(e.value()),
                                required: true,
                                min: "0",
                            }
                        }
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Currency" }
                            select {
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                value: "{currency}",
                                onchange: move |e| currency.set(e.value()),
                                option { value: "USD", "USD" }
                                option { value: "EUR", "EUR" }
                                option { value: "GBP", "GBP" }
                            }
                        }
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Interval" }
                            select {
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                value: "{interval}",
                                onchange: move |e| interval.set(e.value()),
                                option { value: "month", "Monthly" }
                                option { value: "year", "Yearly" }
                            }
                        }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Trial Days" }
                            input {
                                r#type: "number",
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                placeholder: "0",
                                value: "{trial_days}",
                                oninput: move |e| trial_days.set(e.value()),
                                min: "0",
                            }
                        }
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Sort Order" }
                            input {
                                r#type: "number",
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                placeholder: "0",
                                value: "{sort_order}",
                                oninput: move |e| sort_order.set(e.value()),
                                min: "0",
                            }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: is_active(),
                            onchange: move |_| is_active.set(!is_active()),
                            class: "rounded border-border",
                        }
                        label { class: "text-sm font-medium", "Active" }
                    }
                    div { class: "flex gap-3 pt-4",
                        Button {
                            variant: ButtonVariant::Default,
                            r#type: "submit",
                            disabled: saving(),
                            if saving() { "Saving..." } else { "Save Changes" }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            r#type: "button",
                            onclick: move |_| { nav.push(Route::BillingPlansListScreen {}); },
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
