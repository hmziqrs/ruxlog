use crate::seo::{use_static_seo, SeoHead};
use dioxus::prelude::*;

#[component]
pub fn AboutScreen() -> Element {
    let seo_metadata = use_static_seo("about");

    rsx! {
        SeoHead { metadata: seo_metadata }

        div { class: "min-h-screen bg-background/10",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-4xl font-bold mb-6", "About Ruxlog" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { class: "text-lg mb-4",
                        "Welcome to Ruxlog - a modern blogging platform built from scratch."
                    }
                    // Add more content here as needed
                }
            }
        }
    }
}
