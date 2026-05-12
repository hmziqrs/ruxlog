use dioxus::prelude::*;

use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::store::{use_billing, Payment};

#[component]
pub fn RefundsListScreen() -> Element {
    let billing = use_billing();

    use_effect(move || {
        spawn(async move {
            billing.list_payments().await;
        });
    });

    let payments_list = billing.payments_list.read();
    let payments_loading = payments_list.is_loading();
    let payments = payments_list.data.clone().unwrap_or_default();
    let is_failed = payments_list.is_failed();

    // Filter payments to only show refunded ones
    let refunds: Vec<&Payment> = payments
        .iter()
        .filter(|p| p.status == "refunded")
        .collect();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Refunds" }
                    p { class: "text-sm text-muted-foreground", "View refunded payments and transaction details." }
                }
            }

            if is_failed {
                div { class: "text-center py-8",
                    p { class: "text-destructive mb-4", "Failed to load refunds" }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let billing = billing;
                            spawn(async move { billing.list_payments().await; });
                        },
                        "Retry"
                    }
                }
            } else if payments_loading && refunds.is_empty() {
                div { class: "flex items-center justify-center py-20",
                    div { class: "animate-pulse text-muted-foreground", "Loading refunds..." }
                }
            } else if refunds.is_empty() {
                div { class: "text-center py-20 text-muted-foreground", "No refunds yet" }
            } else {
                div { class: "rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden",
                    table { class: "w-full",
                        thead { class: "bg-muted/50",
                            tr {
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "ID" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Payment Ref" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "User" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Amount" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Provider" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Description" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Date" }
                            }
                        }
                        tbody {
                            for payment in refunds.iter() {
                                { refund_row(payment) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn refund_row(payment: &Payment) -> Element {
    let amount_display = format!(
        "{}{:.2}",
        payment.currency,
        payment.amount_cents as f64 / 100.0
    );
    let created = crate::utils::dates::format_short_date_dt(&payment.created_at);

    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground", "#{payment.id}" }
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground font-mono",
                {payment.provider_payment_id.clone().unwrap_or_else(|| "—".to_string())}
            }
            td { class: "py-2 px-3 text-xs md:text-sm", "#{payment.user_id}" }
            td { class: "py-2 px-3 text-xs md:text-sm font-medium", "{amount_display}" }
            td { class: "py-2 px-3 text-xs md:text-sm", "{payment.provider}" }
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground max-w-xs truncate",
                {payment.description.clone().unwrap_or_else(|| "—".to_string())}
            }
            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap",
                Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Refunded" }
            }
        }
    }
}
