use crate::config::BRAND;
use crate::seo::{use_static_seo, SeoHead};
use dioxus::prelude::*;

#[component]
pub fn PrivacyPolicyScreen() -> Element {
    let seo_metadata = use_static_seo("privacy");

    rsx! {
        SeoHead { metadata: seo_metadata }

        div { class: "min-h-screen bg-background",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-4xl font-bold mb-6", "Privacy Policy" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { "At {BRAND.app_name}, we value your privacy." }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "What We Collect" }
                    p { "We use Firebase Analytics to understand how visitors interact with this blog. No personal data is collected—only anonymous usage statistics:" }
                    ul {
                        li { "Page views" }
                        li { "Time spent on pages" }
                        li { "Scroll depth" }
                        li { "Browser type and device" }
                    }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "How We Use It" }
                    p { "Analytics data helps us:" }
                    ul {
                        li { "Understand which content resonates with readers" }
                        li { "Identify technical issues" }
                        li { "Improve overall user experience" }
                    }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Third Parties" }
                    p { "Firebase Analytics is provided by Google. Data is processed according to "
                        a {
                            href: "https://policies.google.com/privacy",
                            target: "_blank",
                            rel: "noopener",
                            "Google's privacy policy"
                        }
                        "."
                    }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Your Choices" }
                    p { "You can opt-out of analytics by:" }
                    ul {
                        li { "Disabling cookies in your browser" }
                        li { "Using browser extensions that block analytics scripts" }
                    }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "No Data Selling" }
                    p { "We never sell or share your data with third parties for advertising or marketing purposes." }

                    p { class: "mt-8 text-sm text-muted-foreground", "Last updated: {BRAND.copyright_year}" }
                }
            }
        }
    }
}
