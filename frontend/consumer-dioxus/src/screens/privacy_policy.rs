use dioxus::prelude::*;

#[component]
pub fn PrivacyPolicyScreen() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            div { class: "max-w-3xl mx-auto",
                h1 { class: "text-4xl font-bold mb-6", "Privacy Policy" }
                div { class: "prose dark:prose-invert max-w-none text-muted-foreground",
                    p { "Our privacy policy will live here. Check back soon for the full details." }
                }
            }
        }
    }
}
