use dioxus::prelude::*;

#[component]
pub fn ContactScreen() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            div { class: "max-w-3xl mx-auto",
                h1 { class: "text-4xl font-bold mb-6", "Contact Us" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { class: "text-lg text-muted-foreground mb-4",
                        "Get in touch with us."
                    }
                    // Add contact form or information here as needed
                }
            }
        }
    }
}
