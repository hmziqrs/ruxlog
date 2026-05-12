use dioxus::prelude::*;

use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::store::{use_billing, Subscription};

#[component]
pub fn BillingSubscriptionsListScreen() -> Element {
    let billing = use_billing();

    use_effect(move || {
        spawn(async move {
            billing.list_subscriptions().await;
        });
    });

    let subs_list = billing.subscriptions_list.read();
    let subs_loading = subs_list.is_loading();
    let subs = subs_list.data.clone().unwrap_or_default();
    let is_failed = subs_list.is_failed();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Subscriptions" }
                    p { class: "text-sm text-muted-foreground", "View and manage user subscriptions." }
                }
            }

            if is_failed {
                div { class: "text-center py-8",
                    p { class: "text-destructive mb-4", "Failed to load subscriptions" }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let billing = billing;
                            spawn(async move { billing.list_subscriptions().await; });
                        },
                        "Retry"
                    }
                }
            } else if subs_loading && subs.is_empty() {
                div { class: "flex items-center justify-center py-20",
                    div { class: "animate-pulse text-muted-foreground", "Loading..." }
                }
            } else if subs.is_empty() {
                div { class: "text-center py-20 text-muted-foreground", "No subscriptions yet" }
            } else {
                div { class: "rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden",
                    table { class: "w-full",
                        thead { class: "bg-muted/50",
                            tr {
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "ID" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "User" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Plan" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Provider" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Status" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Created" }
                                th { class: "w-12 py-2 px-3", "" }
                            }
                        }
                        tbody {
                            for sub in subs.iter() {
                                {
                                    let sub_id = sub.id;
                                    let status_label = match sub.status.as_str() {
                                        "active" => "Active".to_string(),
                                        "canceled" => "Canceled".to_string(),
                                        "trialing" => "Trialing".to_string(),
                                        other => other.to_string(),
                                    };
                                    let status_class = match sub.status.as_str() {
                                        "active" => "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400",
                                        "canceled" => "bg-red-100 text-red-800 border-red-200 dark:bg-red-900/20 dark:text-red-400",
                                        "trialing" => "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/20 dark:text-blue-400",
                                        _ => "",
                                    };
                                    let is_active = sub.status == "active";
                                    let created = crate::utils::dates::format_short_date_dt(&sub.created_at);

                                    rsx! {
                                        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground", "#{sub.id}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm", "#{sub.user_id}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm", "#{sub.plan_id}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm", "{sub.provider}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm",
                                                if status_class.is_empty() {
                                                    Badge { variant: BadgeVariant::Secondary, "{status_label}" }
                                                } else {
                                                    Badge { class: "{status_class}", "{status_label}" }
                                                }
                                            }
                                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap", "{created}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm",
                                                if is_active {
                                                    Button {
                                                        variant: ButtonVariant::Ghost,
                                                        class: "h-8 text-red-600 hover:text-red-700 text-xs",
                                                        onclick: move |_| {
                                                            let billing = billing;
                                                            let id = sub_id;
                                                            spawn(async move { billing.cancel_subscription(id).await; });
                                                        },
                                                        "Cancel"
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
}
