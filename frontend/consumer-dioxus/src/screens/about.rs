use dioxus::prelude::*;

#[component]
pub fn AboutScreen() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            div { class: "max-w-3xl mx-auto",
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
