use dioxus::prelude::*;

use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::card::Card;
use oxui::shadcn::checkbox::Checkbox;

#[derive(Debug, Clone)]
struct PaymentProvider {
    name: String,
    description: String,
    env_key: String,
    webhook_path: String,
}

fn payment_providers() -> Vec<PaymentProvider> {
    vec![
        PaymentProvider {
            name: "Stripe".to_string(),
            description: "Accept credit cards, Apple Pay, Google Pay, and more via Stripe Checkout."
                .to_string(),
            env_key: "STRIPE_SECRET_KEY".to_string(),
            webhook_path: "/billing/v1/webhooks/stripe".to_string(),
        },
        PaymentProvider {
            name: "Polar".to_string(),
            description: "All-in-one monetization platform for SaaS, digital products, and memberships."
                .to_string(),
            env_key: "POLAR_ACCESS_TOKEN".to_string(),
            webhook_path: "/billing/v1/webhooks/polar".to_string(),
        },
        PaymentProvider {
            name: "Lemon Squeezy".to_string(),
            description: "Global merchant of record with built-in tax compliance and licensing."
                .to_string(),
            env_key: "LEMONSQUEEZY_API_KEY".to_string(),
            webhook_path: "/billing/v1/webhooks/lemon_squeezy".to_string(),
        },
        PaymentProvider {
            name: "Paddle".to_string(),
            description: "Complete payments, tax, and subscription management platform.".to_string(),
            env_key: "PADDLE_VENDOR_ID".to_string(),
            webhook_path: "/billing/v1/webhooks/paddle".to_string(),
        },
        PaymentProvider {
            name: "Crypto".to_string(),
            description: "Accept cryptocurrency payments via on-chain transactions.".to_string(),
            env_key: "CRYPTO_WALLET_ADDRESS".to_string(),
            webhook_path: "/billing/v1/webhooks/crypto".to_string(),
        },
    ]
}

#[component]
pub fn BillingSettingsScreen() -> Element {
    let providers = payment_providers();
    let mut currency = use_signal(|| "USD".to_string());
    let mut trial_days = use_signal(|| "14".to_string());
    let mut tax_rate = use_signal(|| "0".to_string());

    rsx! {
        div { class: "space-y-8",
            // -- Provider toggles section --
            div { class: "space-y-4",
                div {
                    h2 { class: "text-lg font-semibold", "Payment Providers" }
                    p { class: "text-sm text-muted-foreground",
                        "Enable or disable payment providers. Providers are activated by setting their environment variables on the server."
                    }
                }

                div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                    for provider in providers.iter() {
                        {
                            let name = provider.name.clone();
                            let description = provider.description.clone();
                            let env_key = provider.env_key.clone();
                            let webhook_path = provider.webhook_path.clone();

                            rsx! {
                                ProviderCard {
                                    name,
                                    description,
                                    env_key,
                                    webhook_path,
                                }
                            }
                        }
                    }
                }
            }

            // -- General settings section --
            div { class: "space-y-4",
                div {
                    h2 { class: "text-lg font-semibold", "General Settings" }
                    p { class: "text-sm text-muted-foreground",
                        "Configure default billing behavior for your application."
                    }
                }

                Card { class: "p-6 space-y-6",
                    // Default currency
                    div { class: "grid gap-2 max-w-[300px]",
                        label { class: "text-sm font-medium", "Default Currency" }
                        select {
                            class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                            value: "{currency}",
                            onchange: move |e| currency.set(e.value()),
                            option { value: "USD", "USD - US Dollar" }
                            option { value: "EUR", "EUR - Euro" }
                            option { value: "GBP", "GBP - British Pound" }
                            option { value: "CAD", "CAD - Canadian Dollar" }
                            option { value: "AUD", "AUD - Australian Dollar" }
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Currency used for new plans and displayed amounts."
                        }
                    }

                    // Trial period days
                    div { class: "grid gap-2 max-w-[300px]",
                        label { class: "text-sm font-medium", "Trial Period (days)" }
                        input {
                            r#type: "number",
                            class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                            placeholder: "14",
                            value: "{trial_days}",
                            oninput: move |e| trial_days.set(e.value()),
                            min: "0",
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Number of free trial days for new subscriptions. Set to 0 to disable trials."
                        }
                    }

                    // Tax rate
                    div { class: "grid gap-2 max-w-[300px]",
                        label { class: "text-sm font-medium", "Tax Rate (%)" }
                        input {
                            r#type: "number",
                            class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                            placeholder: "0",
                            value: "{tax_rate}",
                            oninput: move |e| tax_rate.set(e.value()),
                            min: "0",
                            max: "100",
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Default tax rate applied to invoices. Can be overridden per plan."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProviderCard(
    name: String,
    description: String,
    env_key: String,
    webhook_path: String,
) -> Element {
    let mut enabled = use_signal(|| false);

    rsx! {
        Card { class: "p-5 space-y-4",
            // Header: name + toggle
            div { class: "flex items-center justify-between gap-3",
                h3 { class: "text-sm font-semibold", "{name}" }
                Checkbox {
                    checked: *enabled.read(),
                    onchange: move |checked| enabled.set(checked),
                }
            }

            // Description
            p { class: "text-xs text-muted-foreground leading-relaxed", "{description}" }

            // Status badge
            div { class: "flex items-center gap-2",
                Badge {
                    variant: if *enabled.read() { BadgeVariant::Default } else { BadgeVariant::Secondary },
                    class: if *enabled.read() {
                        "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400"
                    } else {
                        "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400"
                    },
                    if *enabled.read() { "Enabled" } else { "Disabled" }
                }
                span { class: "text-xs text-muted-foreground", "Check server for live status" }
            }

            // Env key
            div { class: "space-y-1",
                p { class: "text-xs text-muted-foreground",
                    span { class: "font-medium", "Env: " }
                    code { class: "text-xs bg-muted px-1.5 py-0.5 rounded font-mono", "{env_key}" }
                }
            }

            // Webhook URL
            div { class: "space-y-1",
                p { class: "text-xs text-muted-foreground",
                    span { class: "font-medium", "Webhook: " }
                    code { class: "text-xs bg-muted px-1.5 py-0.5 rounded font-mono", "{webhook_path}" }
                }
            }

            // Test webhook button
            Button {
                variant: ButtonVariant::Outline,
                disabled: !*enabled.read(),
                class: "w-full mt-2",
                "Test Webhook"
            }
        }
    }
}
