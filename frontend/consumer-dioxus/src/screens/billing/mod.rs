use crate::config::BRAND;
use crate::router::Route;
use crate::seo::{breadcrumb_schema, SeoHead, SeoMetadataBuilder, StructuredData};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdCalendar, LdCheck, LdCreditCard};
use hmziq_dioxus_free_icons::Icon;

#[derive(Debug, Clone, serde::Deserialize)]
struct Subscription {
    id: i32,
    plan_name: String,
    status: String,
    provider: String,
    current_period_start: Option<String>,
    current_period_end: Option<String>,
    cancel_at_period_end: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Payment {
    id: i32,
    amount_cents: i32,
    currency: String,
    status: String,
    provider: String,
    description: Option<String>,
    created_at: String,
}

#[component]
pub fn BillingScreen() -> Element {
    let mut subscriptions = use_signal(Vec::<Subscription>::new);
    let mut payments = use_signal(Vec::<Payment>::new);
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let seo_metadata = SeoMetadataBuilder::new()
        .title("Billing")
        .description(&format!("Manage your subscription on {}", BRAND.app_name))
        .canonical("/billing")
        .build();

    use_effect(move || {
        spawn(async move {
            let sub_resp = oxcore::http::get("/billing/v1/subscriptions").send().await;
            let pay_resp = oxcore::http::get("/billing/v1/payments").send().await;

            match (sub_resp, pay_resp) {
                (Ok(sr), Ok(pr)) => {
                    if (200..300).contains(&sr.status()) {
                        if let Ok(data) = sr.json::<Vec<Subscription>>().await {
                            subscriptions.set(data);
                        }
                    }
                    if (200..300).contains(&pr.status()) {
                        if let Ok(data) = pr.json::<Vec<Payment>>().await {
                            payments.set(data);
                        }
                    }
                    loading.set(false);
                }
                _ => {
                    error_msg.set(Some("Unable to load billing information.".to_string()));
                    loading.set(false);
                }
            }
        });
    });

    let active_sub = subscriptions()
        .iter()
        .find(|s| s.status == "active")
        .cloned();

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Billing", "/billing"),
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 max-w-4xl",
                div { class: "mb-8",
                    h1 { class: "text-3xl font-bold mb-2", "Billing & Subscription" }
                    p { class: "text-muted-foreground",
                        "Manage your subscription and view payment history."
                    }
                }

                if loading() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "animate-pulse text-muted-foreground", "Loading..." }
                    }
                } else if let Some(err) = error_msg() {
                    div { class: "text-center py-20",
                        p { class: "text-muted-foreground", "{err}" }
                    }
                } else {
                    // Active subscription card
                    if let Some(sub) = active_sub {
                        div { class: "rounded-xl border border-border bg-card p-6 mb-8",
                            div { class: "flex items-start justify-between mb-4",
                                div {
                                    div { class: "flex items-center gap-2 mb-1",
                                        Icon { icon: LdCheck, class: "w-5 h-5 text-green-500" }
                                        h2 { class: "text-xl font-semibold", "Active Subscription" }
                                    }
                                    p { class: "text-muted-foreground text-sm",
                                        "Your subscription is active"
                                    }
                                }
                                span { class: "px-3 py-1 rounded-full text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400",
                                    "{sub.status}"
                                }
                            }
                            div { class: "grid sm:grid-cols-2 gap-4 text-sm",
                                div {
                                    p { class: "text-muted-foreground mb-1", "Plan" }
                                    p { class: "font-medium", "{sub.plan_name}" }
                                }
                                div {
                                    p { class: "text-muted-foreground mb-1", "Provider" }
                                    p { class: "font-medium capitalize", "{sub.provider}" }
                                }
                                if let Some(end) = &sub.current_period_end {
                                    div {
                                        p { class: "text-muted-foreground mb-1", "Renews" }
                                        p { class: "font-medium", "{end}" }
                                    }
                                }
                                if sub.cancel_at_period_end {
                                    div {
                                        p { class: "text-muted-foreground mb-1", "Status" }
                                        p { class: "font-medium text-amber-600", "Cancels at period end" }
                                    }
                                }
                            }
                            if !sub.cancel_at_period_end {
                                div { class: "mt-4 pt-4 border-t border-border",
                                    Link {
                                        to: Route::PricingScreen {},
                                        class: "text-sm text-primary hover:underline",
                                        "Change plan"
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "rounded-xl border border-border bg-card p-6 mb-8 text-center",
                            Icon { icon: LdCreditCard, class: "w-10 h-10 mx-auto text-muted-foreground mb-3" }
                            h2 { class: "text-xl font-semibold mb-2", "No Active Subscription" }
                            p { class: "text-muted-foreground mb-4",
                                "Subscribe to a plan to unlock premium content."
                            }
                            Link {
                                to: Route::PricingScreen {},
                                class: "inline-flex items-center justify-center px-6 py-2.5 rounded-lg bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity",
                                "View Plans"
                            }
                        }
                    }

                    // Payment history
                    if !payments().is_empty() {
                        div { class: "rounded-xl border border-border bg-card",
                            div { class: "p-6 border-b border-border",
                                h2 { class: "text-lg font-semibold", "Payment History" }
                            }
                            div { class: "divide-y divide-border",
                                for payment in payments().iter() {
                                    {
                                        let amount = format!(
                                            "{}{:.2}",
                                            payment.currency,
                                            payment.amount_cents as f64 / 100.0
                                        );
                                        let status_color = match payment.status.as_str() {
                                            "succeeded" | "completed" => "text-green-600",
                                            "pending" => "text-amber-600",
                                            _ => "text-muted-foreground",
                                        };

                                        rsx! {
                                            div { class: "px-6 py-4 flex items-center justify-between",
                                                div { class: "flex items-center gap-3",
                                                    Icon { icon: LdCalendar, class: "w-4 h-4 text-muted-foreground" }
                                                    div {
                                                        p { class: "font-medium text-sm", "{amount}" }
                                                        if let Some(desc) = &payment.description {
                                                            p { class: "text-xs text-muted-foreground", "{desc}" }
                                                        }
                                                    }
                                                }
                                                div { class: "text-right",
                                                    p { class: "text-xs text-muted-foreground", "{payment.created_at}" }
                                                    p { class: "text-xs font-medium {status_color} capitalize", "{payment.status}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
