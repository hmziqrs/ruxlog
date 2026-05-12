use dioxus::prelude::*;
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::store::{use_billing, PlansAddPayload};

use crate::router::Route;

#[component]
pub fn BillingPlanAddScreen() -> Element {
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

    let on_name_input = move |e: Event<FormData>| {
        let val = e.value();
        let slug_val = val
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let slug_val = slug_val
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        name.set(val);
        slug.set(slug_val);
    };

    let on_submit = move |_| {
        let name_val = name.read().clone();
        let slug_val = slug.read().clone();
        let desc_val = description.read().clone();
        let price_val: i32 = price_cents.read().parse().unwrap_or(0);
        let curr_val = currency.read().clone();
        let int_val = interval.read().clone();
        let trial_val = trial_days.read().parse().ok();
        let sort_val = sort_order.read().parse().ok();

        if name_val.trim().is_empty() || slug_val.trim().is_empty() || price_val <= 0 {
            return;
        }

        let payload = PlansAddPayload {
            name: name_val,
            slug: slug_val,
            description: if desc_val.trim().is_empty() {
                None
            } else {
                Some(desc_val)
            },
            price_cents: price_val,
            currency: curr_val,
            interval: int_val,
            trial_days: trial_val,
            features: None,
            is_active: Some(is_active()),
            sort_order: sort_val,
        };

        spawn(async move {
            saving.set(true);
            billing.add_plan(payload).await;
            saving.set(false);
            nav.push(Route::BillingPlansListScreen {});
        });
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Add Plan" }
                    p { class: "text-sm text-muted-foreground", "Create a new subscription plan." }
                }
            }

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
                        label { class: "text-sm font-medium", "Slug *" }
                        input {
                            class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm font-mono",
                            placeholder: "pro-plan",
                            value: "{slug}",
                            oninput: move |e| slug.set(e.value()),
                            required: true,
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
                        if saving() { "Creating..." } else { "Create Plan" }
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
