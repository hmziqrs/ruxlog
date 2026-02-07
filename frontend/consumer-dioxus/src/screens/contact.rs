use crate::seo::{use_static_seo, SeoHead};
use dioxus::prelude::*;

#[component]
pub fn ContactScreen() -> Element {
    let seo_metadata = use_static_seo("contact");

    rsx! {
        SeoHead { metadata: seo_metadata }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-4xl font-bold mb-6", "Contact Us" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { class: "text-lg mb-4",
                        "Get in touch with us."
                    }
                    // Add contact form or information here as needed
                }
            }
        }
    }
}
