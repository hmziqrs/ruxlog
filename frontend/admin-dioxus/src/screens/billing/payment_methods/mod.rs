use dioxus::prelude::*;

use oxui::shadcn::badge::{Badge, BadgeVariant};

#[derive(Debug, Clone)]
struct ProviderInfo {
    name: String,
    slug: String,
    description: String,
    env_key: String,
}

fn configured_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            name: "Stripe".to_string(),
            slug: "stripe".to_string(),
            description: "Accept credit cards, Apple Pay, Google Pay, and more via Stripe Checkout.".to_string(),
            env_key: "STRIPE_SECRET_KEY".to_string(),
        },
        ProviderInfo {
            name: "Polar".to_string(),
            slug: "polar".to_string(),
            description: "All-in-one monetization platform for SaaS, digital products, and memberships.".to_string(),
            env_key: "POLAR_ACCESS_TOKEN".to_string(),
        },
        ProviderInfo {
            name: "Lemon Squeezy".to_string(),
            slug: "lemon_squeezy".to_string(),
            description: "Global merchant of record with built-in tax compliance and licensing.".to_string(),
            env_key: "LEMONSQUEEZY_API_KEY".to_string(),
        },
        ProviderInfo {
            name: "Paddle".to_string(),
            slug: "paddle".to_string(),
            description: "Complete payments, tax, and subscription management platform.".to_string(),
            env_key: "PADDLE_VENDOR_ID".to_string(),
        },
        ProviderInfo {
            name: "Crypto".to_string(),
            slug: "crypto".to_string(),
            description: "Accept cryptocurrency payments via on-chain transactions.".to_string(),
            env_key: "CRYPTO_WALLET_ADDRESS".to_string(),
        },
    ]
}

#[component]
pub fn PaymentMethodsScreen() -> Element {
    let providers = configured_providers();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold", "Payment Methods" }
                    p { class: "text-sm text-muted-foreground",
                        "Overview of configured payment providers and their webhook endpoints."
                    }
                }
            }
            p { class: "text-sm text-muted-foreground",
                "Payment providers are configured via environment variables on your server. "
                "Set the required keys to enable a provider, then configure webhook URLs in the provider dashboard to point to your Ruxlog instance."
            }

            div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                for provider in providers.iter() {
                    {
                        let slug = provider.slug.clone();
                        let webhook_path = format!("/billing/v1/webhooks/{}", slug);
                        let env_key = provider.env_key.clone();

                        rsx! {
                            div { class: "rounded-lg border border-zinc-200 dark:border-zinc-800 p-5 space-y-3",
                                div { class: "flex items-center justify-between",
                                    h3 { class: "text-sm font-semibold", "{provider.name}" }
                                    Badge {
                                        variant: BadgeVariant::Secondary,
                                        class: "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400",
                                        "Check server"
                                    }
                                }
                                p { class: "text-xs text-muted-foreground leading-relaxed", "{provider.description}" }
                                div { class: "space-y-1",
                                    p { class: "text-xs text-muted-foreground",
                                        span { class: "font-medium", "Env: " }
                                        code { class: "text-xs bg-muted px-1.5 py-0.5 rounded font-mono", "{env_key}" }
                                    }
                                    p { class: "text-xs text-muted-foreground",
                                        span { class: "font-medium", "Webhook: " }
                                        code { class: "text-xs bg-muted px-1.5 py-0.5 rounded font-mono",
                                            "{webhook_path}"
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
