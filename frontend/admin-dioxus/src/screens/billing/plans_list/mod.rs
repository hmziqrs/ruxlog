use dioxus::prelude::*;

use crate::components::table::list_empty_state::ListEmptyState;
use crate::router::Route;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdEllipsis, Icon};
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use ruxlog_shared::store::{use_billing, Plan};

#[component]
pub fn BillingPlansListScreen() -> Element {
    let nav = use_navigator();
    let billing = use_billing();

    use_effect(move || {
        spawn(async move {
            billing.list_plans().await;
        });
    });

    let plans_list = billing.plans_list.read();
    let plans_loading = plans_list.is_loading();
    let plans = plans_list.data.clone().unwrap_or_default();
    let is_failed = plans_list.is_failed();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Billing Plans" }
                    p { class: "text-sm text-muted-foreground", "Manage subscription plans, pricing, and features." }
                }
                Button {
                    onclick: move |_| { nav.push(Route::BillingPlanAddScreen {}); },
                    "New Plan"
                }
            }

            if is_failed {
                div { class: "text-center py-8",
                    p { class: "text-destructive mb-4", "Failed to load plans" }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let billing = billing;
                            spawn(async move { billing.list_plans().await; });
                        },
                        "Retry"
                    }
                }
            } else if plans_loading && plans.is_empty() {
                div { class: "flex items-center justify-center py-20",
                    div { class: "animate-pulse text-muted-foreground", "Loading plans..." }
                }
            } else if plans.is_empty() {
                ListEmptyState {
                    title: "No plans found".to_string(),
                    description: "Create your first subscription plan to get started.".to_string(),
                    clear_label: "Refresh".to_string(),
                    create_label: "Create your first plan".to_string(),
                    on_clear: move |_| {
                        let billing = billing;
                        spawn(async move { billing.list_plans().await; });
                    },
                    on_create: move |_| { nav.push(Route::BillingPlanAddScreen {}); },
                }
            } else {
                div { class: "rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden",
                    table { class: "w-full",
                        thead { class: "bg-muted/50",
                            tr {
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Name" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Slug" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Price" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Interval" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Trial" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Status" }
                                th { class: "w-12 py-2 px-3", "" }
                            }
                        }
                        tbody {
                            for plan in plans.iter() {
                                {
                                    let plan_id = plan.id;
                                    let price_display = format!("{}{:.2}", plan.currency, plan.price_cents as f64 / 100.0);
                                    let interval_display = match plan.interval.as_str() {
                                        "month" => "Monthly".to_string(),
                                        "year" => "Yearly".to_string(),
                                        other => other.to_string(),
                                    };
                                    let trial_display = if plan.trial_days > 0 {
                                        format!("{} days", plan.trial_days)
                                    } else {
                                        "—".to_string()
                                    };

                                    rsx! {
                                        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                                            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap",
                                                span { class: "font-medium", "{plan.name}" }
                                            }
                                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap font-mono", "{plan.slug}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap", "{price_display}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap", "{interval_display}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap", "{trial_display}" }
                                            td { class: "py-2 px-3 text-xs md:text-sm",
                                                if plan.is_active {
                                                    Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Active" }
                                                } else {
                                                    Badge { variant: BadgeVariant::Secondary, class: "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400", "Inactive" }
                                                }
                                            }
                                            td { class: "py-2 px-3 text-xs md:text-sm",
                                                DropdownMenu {
                                                    DropdownMenuTrigger {
                                                        Button { variant: ButtonVariant::Ghost, class: "h-8 w-8 p-0 bg-transparent hover:bg-muted/50",
                                                            div { class: "w-4 h-4", Icon { icon: LdEllipsis {} } }
                                                        }
                                                    }
                                                    DropdownMenuContent { class: "bg-background border-zinc-200 dark:border-zinc-800",
                                                        DropdownMenuItem { onclick: move |_| {
                                                                nav.push(Route::BillingPlanEditScreen { id: plan_id });
                                                            }, "Edit" }
                                                        DropdownMenuItem { class: "text-red-600", onclick: move |_| {
                                                                let billing = billing;
                                                                let id = plan_id;
                                                                spawn(async move { billing.remove_plan(id).await; });
                                                            }, "Delete" }
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
