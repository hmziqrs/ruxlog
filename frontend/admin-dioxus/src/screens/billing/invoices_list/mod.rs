use dioxus::prelude::*;

use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::store::{use_billing, Invoice};

#[component]
pub fn BillingInvoicesListScreen() -> Element {
    let billing = use_billing();

    use_effect(move || {
        spawn(async move {
            billing.list_invoices().await;
        });
    });

    let invoices_list = billing.invoices_list.read();
    let invoices_loading = invoices_list.is_loading();
    let invoices = invoices_list.data.clone().unwrap_or_default();
    let is_failed = invoices_list.is_failed();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Invoices" }
                    p { class: "text-sm text-muted-foreground", "View and manage billing invoices." }
                }
            }

            if is_failed {
                div { class: "text-center py-8",
                    p { class: "text-destructive mb-4", "Failed to load invoices" }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let billing = billing;
                            spawn(async move { billing.list_invoices().await; });
                        },
                        "Retry"
                    }
                }
            } else if invoices_loading && invoices.is_empty() {
                div { class: "flex items-center justify-center py-20",
                    div { class: "animate-pulse text-muted-foreground", "Loading..." }
                }
            } else if invoices.is_empty() {
                div { class: "text-center py-20 text-muted-foreground", "No invoices yet" }
            } else {
                div { class: "rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden",
                    table { class: "w-full",
                        thead { class: "bg-muted/50",
                            tr {
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Invoice #" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "User" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Amount" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Currency" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Status" }
                                th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Created" }
                            }
                        }
                        tbody {
                            for invoice in invoices.iter() {
                                { invoice_row(invoice) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn invoice_row(invoice: &Invoice) -> Element {
    let amount_display = format!(
        "{}{:.2}",
        invoice.currency,
        invoice.amount_cents as f64 / 100.0
    );
    let status = invoice.status.clone();

    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "py-2 px-3 text-xs md:text-sm font-medium", "{invoice.invoice_number}" }
            td { class: "py-2 px-3 text-xs md:text-sm", "#{invoice.user_id}" }
            td { class: "py-2 px-3 text-xs md:text-sm font-medium", "{amount_display}" }
            td { class: "py-2 px-3 text-xs md:text-sm", "{invoice.currency}" }
            td { class: "py-2 px-3 text-xs md:text-sm",
                { match status.as_str() {
                    "paid" => rsx! { Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Paid" } },
                    "pending" => rsx! { Badge { class: "bg-yellow-100 text-yellow-800 border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-400", "Pending" } },
                    "failed" | "overdue" => rsx! { Badge { variant: BadgeVariant::Secondary, class: "bg-red-100 text-red-800 border-red-200 dark:bg-red-900/20 dark:text-red-400", "Failed" } },
                    "refunded" => rsx! { Badge { class: "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/20 dark:text-blue-400", "Refunded" } },
                    "void" => rsx! { Badge { variant: BadgeVariant::Secondary, class: "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400", "Void" } },
                    other => rsx! { Badge { variant: BadgeVariant::Secondary, "{other}" } },
                }}
            }
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                "{crate::utils::dates::format_short_date_dt(&invoice.created_at)}"
            }
        }
    }
}
