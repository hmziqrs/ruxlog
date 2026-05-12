use crate::config::BRAND;
use crate::seo::{breadcrumb_schema, use_static_seo, SeoHead, StructuredData};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{
    LdBolt, LdCode, LdGlobe, LdHeart, LdShield, LdTrendingUp,
};
use hmziq_dioxus_free_icons::Icon;

#[component]
pub fn AboutScreen() -> Element {
    let seo_metadata = use_static_seo("about");

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("About", "/about"),
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-4xl",
                // Hero section
                div { class: "text-center mb-16",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-4",
                        "About {BRAND.app_name}"
                    }
                    p { class: "text-xl text-muted-foreground max-w-2xl mx-auto",
                        "{BRAND.tagline}"
                    }
                }

                // Mission section
                div { class: "mb-16",
                    h2 { class: "text-2xl font-bold mb-4", "Our Mission" }
                    div { class: "prose dark:prose-invert max-w-none space-y-4",
                        p {
                            "{BRAND.app_name} is a modern blogging platform built entirely in Rust, \
                            designed for developers who value performance, privacy, and clean design."
                        }
                        p {
                            "We believe blogging tools should be fast, reliable, and respect your readers' privacy. \
                            No tracking scripts from third parties, no bloated JavaScript frameworks, \
                            no compromises on load times."
                        }
                        p {
                            "Every part of the stack — from the Axum backend to the Dioxus frontends — \
                            is written in Rust for memory safety, concurrency, and speed."
                        }
                    }
                }

                // Tech stack section
                div { class: "mb-16",
                    h2 { class: "text-2xl font-bold mb-6", "Built With" }
                    div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                        TechCard {
                            icon: rsx! { Icon { icon: LdBolt, class: "w-6 h-6" } },
                            title: "Axum",
                            description: "High-performance async web framework powering the API",
                        }
                        TechCard {
                            icon: rsx! { Icon { icon: LdCode, class: "w-6 h-6" } },
                            title: "Dioxus",
                            description: "Full-stack Rust UI framework for both admin and consumer apps",
                        }
                        TechCard {
                            icon: rsx! { Icon { icon: LdShield, class: "w-6 h-6" } },
                            title: "SeaORM",
                            description: "Async ORM with compile-time query checking",
                        }
                        TechCard {
                            icon: rsx! { Icon { icon: LdGlobe, class: "w-6 h-6" } },
                            title: "PostgreSQL",
                            description: "Reliable, feature-rich relational database",
                        }
                        TechCard {
                            icon: rsx! { Icon { icon: LdTrendingUp, class: "w-6 h-6" } },
                            title: "Valkey",
                            description: "In-memory data store for caching and sessions",
                        }
                        TechCard {
                            icon: rsx! { Icon { icon: LdHeart, class: "w-6 h-6" } },
                            title: "RustFS",
                            description: "S3-compatible object storage for media files",
                        }
                    }
                }

                // Open source section
                div { class: "mb-16 rounded-xl border border-border bg-card p-8 text-center",
                    h2 { class: "text-2xl font-bold mb-3", "Open Source" }
                    p { class: "text-muted-foreground mb-6 max-w-xl mx-auto",
                        "{BRAND.app_name} is open source. Read the code, file issues, or contribute."
                    }
                    a {
                        href: "{BRAND.repo_url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "inline-flex items-center gap-2 px-6 py-3 rounded-lg bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity",
                        "View on GitHub"
                    }
                }

                // Author section
                div { class: "text-center",
                    h2 { class: "text-2xl font-bold mb-4", "The Author" }
                    div { class: "flex items-center justify-center gap-4",
                        div { class: "w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center text-2xl font-bold text-primary",
                            "H"
                        }
                        div { class: "text-left",
                            a {
                                href: "{BRAND.author_url}",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "text-lg font-semibold hover:underline",
                                "{BRAND.author}"
                            }
                            p { class: "text-muted-foreground text-sm",
                                "Building things with Rust"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TechCard(icon: Element, title: &'static str, description: &'static str) -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-6 hover:shadow-md transition-shadow",
            div { class: "flex items-center gap-3 mb-3",
                div { class: "text-primary", {icon} }
                h3 { class: "font-semibold", "{title}" }
            }
            p { class: "text-sm text-muted-foreground", "{description}" }
        }
    }
}
