use crate::config::BRAND;
use crate::seo::{breadcrumb_schema, use_static_seo, SeoHead, StructuredData};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdBarChart, LdMegaphone, LdTarget, LdUsers};
use hmziq_dioxus_free_icons::Icon;

#[component]
pub fn AdvertiseScreen() -> Element {
    let seo_metadata = use_static_seo("advertise");

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Advertise", "/advertise"),
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-5xl",
                // Hero
                div { class: "text-center mb-16",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-4",
                        "Advertise with {BRAND.app_name}"
                    }
                    p { class: "text-xl text-muted-foreground max-w-2xl mx-auto",
                        "Reach a growing audience of developers, engineers, and tech enthusiasts."
                    }
                }

                // Stats
                div { class: "grid grid-cols-2 md:grid-cols-4 gap-6 mb-16",
                    StatCard { value: "10K+", label: "Monthly Readers" }
                    StatCard { value: "85%", label: "Developer Audience" }
                    StatCard { value: "3.5 min", label: "Avg. Read Time" }
                    StatCard { value: "42%", label: "Return Visitors" }
                }

                // Pricing tiers
                div { class: "mb-16",
                    h2 { class: "text-2xl font-bold text-center mb-8", "Sponsorship Tiers" }
                    div { class: "grid md:grid-cols-3 gap-6",
                        TierCard {
                            name: "Starter",
                            price: "$99",
                            period: "/month",
                            features: vec![
                                "Sidebar banner placement",
                                "Link in monthly newsletter",
                                "Basic analytics report",
                            ],
                        }
                        TierCard {
                            name: "Growth",
                            price: "$249",
                            period: "/month",
                            highlighted: true,
                            features: vec![
                                "All Starter benefits",
                                "In-article banner (below fold)",
                                "Dedicated newsletter mention",
                                "Detailed analytics dashboard",
                            ],
                        }
                        TierCard {
                            name: "Premium",
                            price: "$499",
                            period: "/month",
                            features: vec![
                                "All Growth benefits",
                                "Sponsored post (quarterly)",
                                "Top placement banner",
                                "Social media promotion",
                                "Priority support",
                            ],
                        }
                    }
                }

                // Why advertise
                div { class: "mb-16",
                    h2 { class: "text-2xl font-bold text-center mb-8", "Why {BRAND.app_name}?" }
                    div { class: "grid md:grid-cols-2 gap-6",
                        WhyCard {
                            icon: rsx! { Icon { icon: LdTarget, class: "w-6 h-6" } },
                            title: "Targeted Audience",
                            description: "Our readers are professional developers and engineers actively looking for tools, services, and learning resources.",
                        }
                        WhyCard {
                            icon: rsx! { Icon { icon: LdBarChart, class: "w-6 h-6" } },
                            title: "Transparent Analytics",
                            description: "Get real-time metrics on impressions, clicks, and engagement. No black-box reporting.",
                        }
                        WhyCard {
                            icon: rsx! { Icon { icon: LdUsers, class: "w-6 h-6" } },
                            title: "Engaged Community",
                            description: "High return-visitor rate means your brand gets repeated exposure to a loyal readership.",
                        }
                        WhyCard {
                            icon: rsx! { Icon { icon: LdMegaphone, class: "w-6 h-6" } },
                            title: "Brand Safety",
                            description: "All content is curated and moderated. Your ads appear alongside quality technical content.",
                        }
                    }
                }

                // CTA
                div { class: "rounded-xl border border-border bg-card p-8 text-center",
                    h2 { class: "text-2xl font-bold mb-3", "Ready to Get Started?" }
                    p { class: "text-muted-foreground mb-6 max-w-lg mx-auto",
                        "Contact us to discuss custom sponsorship packages or to get a detailed media kit."
                    }
                    div { class: "flex items-center justify-center gap-4",
                        Link {
                            to: crate::router::Route::ContactScreen {},
                            class: "inline-flex items-center gap-2 px-6 py-3 rounded-lg bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity",
                            "Contact Us"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatCard(value: &'static str, label: &'static str) -> Element {
    rsx! {
        div { class: "text-center p-4 rounded-lg border border-border bg-card",
            div { class: "text-3xl font-bold text-primary mb-1", "{value}" }
            div { class: "text-sm text-muted-foreground", "{label}" }
        }
    }
}

#[component]
fn WhyCard(icon: Element, title: &'static str, description: &'static str) -> Element {
    rsx! {
        div { class: "flex items-start gap-4 p-6 rounded-lg border border-border bg-card",
            div { class: "text-primary mt-1 shrink-0", {icon} }
            div {
                h3 { class: "font-semibold mb-1", "{title}" }
                p { class: "text-sm text-muted-foreground", "{description}" }
            }
        }
    }
}

#[component]
fn TierCard(
    name: &'static str,
    price: &'static str,
    period: &'static str,
    features: Vec<&'static str>,
    #[props(default = false)] highlighted: bool,
) -> Element {
    let border_class = if highlighted {
        "border-primary shadow-lg scale-[1.02]"
    } else {
        "border-border"
    };

    let badge = if highlighted {
        rsx! {
            div { class: "absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full bg-primary text-primary-foreground text-xs font-medium",
                "Popular"
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "relative rounded-xl {border_class} bg-card p-6 flex flex-col border",
            {badge}
            h3 { class: "text-lg font-semibold mb-2", "{name}" }
            div { class: "mb-6",
                span { class: "text-3xl font-bold", "{price}" }
                span { class: "text-muted-foreground", "{period}" }
            }
            ul { class: "space-y-3 flex-1 mb-6",
                for feature in features.iter() {
                    li { class: "flex items-start gap-2 text-sm",
                        span { class: "text-primary mt-0.5 shrink-0", "+" }
                        span { "{feature}" }
                    }
                }
            }
            Link {
                to: crate::router::Route::ContactScreen {},
                class: if highlighted {
                    "inline-flex items-center justify-center px-4 py-2 rounded-lg bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity"
                } else {
                    "inline-flex items-center justify-center px-4 py-2 rounded-lg border border-border font-medium hover:bg-muted/50 transition-colors"
                },
                "Get Started"
            }
        }
    }
}
