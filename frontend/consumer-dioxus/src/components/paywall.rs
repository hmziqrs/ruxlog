use dioxus::prelude::*;

/// Paywall overlay shown when a user tries to access premium content without a subscription.
#[component]
pub fn PaywallOverlay(access_type: String) -> Element {
    let message = match access_type.as_str() {
        "paid" => "This post requires a one-time purchase to read.",
        "subscriber_only" => "This post is available to subscribers only.",
        _ => "This post requires a subscription to unlock full access.",
    };

    #[cfg(feature = "consumer-auth")]
    let sign_in_link: Option<Element> = Some(rsx! {
        Link {
            to: crate::router::Route::LoginScreen {},
            class: "px-6 py-2.5 rounded-lg border border-border font-medium hover:bg-muted/50 transition-colors",
            "Sign In"
        }
    });

    #[cfg(not(feature = "consumer-auth"))]
    let sign_in_link: Option<Element> = None;

    rsx! {
        div { class: "relative",
            div { class: "absolute inset-0 backdrop-blur-md bg-background/60 z-10 flex items-center justify-center",
                div { class: "max-w-md text-center p-8 rounded-xl border border-border bg-card shadow-lg",
                    h2 { class: "text-2xl font-bold mb-2", "Premium Content" }
                    p { class: "text-muted-foreground mb-6", "{message}" }
                    div { class: "flex flex-col sm:flex-row gap-3 justify-center",
                        Link {
                            to: crate::router::Route::PricingScreen {},
                            class: "px-6 py-2.5 rounded-lg bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity",
                            "View Plans"
                        }
                        {sign_in_link}
                    }
                }
            }
        }
    }
}
