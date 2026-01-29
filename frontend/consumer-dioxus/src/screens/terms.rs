use crate::config::BRAND;
use crate::seo::{use_static_seo, SeoHead};
use dioxus::prelude::*;

#[component]
pub fn TermsScreen() -> Element {
    let seo_metadata = use_static_seo("terms");

    rsx! {
        SeoHead { metadata: seo_metadata }

        div { class: "min-h-screen bg-background",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-4xl font-bold mb-6", "Terms of Service" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { "Welcome to {BRAND.app_name}. By accessing this website, you agree to these terms." }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Content" }
                    p { "All content on this website is owned by {BRAND.author} and protected by copyright laws. Articles, code snippets, and media may not be reproduced without permission." }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Website Use" }
                    p { "You may browse and read content for personal use. You must not:" }
                    ul {
                        li { "Scrape or harvest data without permission" }
                        li { "Use the site for unlawful purposes" }
                        li { "Attempt to circumvent security measures" }
                    }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Analytics" }
                    p { "We use Firebase Analytics to understand how visitors use our site. See our "
                        a { href: "/privacy", "Privacy Policy" }
                        " for details."
                    }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Disclaimer" }
                    p { "Content is provided \"as is\" without warranties. Opinions expressed are those of the author and do not constitute professional advice." }

                    h2 { class: "text-2xl font-semibold mt-8 mb-4", "Changes" }
                    p { "These terms may be updated at any time. Continued use of the site constitutes acceptance of changes." }

                    p { class: "mt-8 text-sm text-muted-foreground", "Last updated: {BRAND.copyright_year}" }
                }
            }
        }
    }
}
