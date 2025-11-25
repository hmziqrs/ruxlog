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

#[component]
pub fn TermsScreen() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            div { class: "max-w-3xl mx-auto",
                h1 { class: "text-4xl font-bold mb-6", "Terms of Service" }
                div { class: "prose dark:prose-invert max-w-none text-muted-foreground",
                    p { "Our terms of service will live here. Check back soon for the full details." }
                }
            }
        }
    }
}

#[component]
pub fn AdvertiseScreen() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            div { class: "max-w-3xl mx-auto",
                h1 { class: "text-4xl font-bold mb-6", "Advertise with Ruxlog" }
                div { class: "prose dark:prose-invert max-w-none text-muted-foreground",
                    p { "Advertising options will be published here. Get in touch to learn more." }
                }
            }
        }
    }
}
