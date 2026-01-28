use dioxus::prelude::*;

#[component]
pub fn PrivacyPolicyScreen() -> Element {
    rsx! {
        div { class: "min-h-screen bg-background",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-4xl font-bold mb-6", "Privacy Policy" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { "Our privacy policy will live here. Check back soon for the full details." }
                }
            }
        }
    }
}
