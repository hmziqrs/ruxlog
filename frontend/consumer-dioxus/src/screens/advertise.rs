use dioxus::prelude::*;

#[component]
pub fn AdvertiseScreen() -> Element {
    rsx! {
        div { class: "min-h-screen bg-background",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-4xl font-bold mb-6", "Advertise with Ruxlog" }
                div { class: "prose dark:prose-invert max-w-none",
                    p { "Advertising options will be published here. Get in touch to learn more." }
                }
            }
        }
    }
}
