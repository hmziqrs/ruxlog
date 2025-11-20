use dioxus::prelude::*;

#[component]
pub fn ProfileScreen() -> Element {
    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8",
                h1 { class: "text-2xl font-bold mb-6", "Profile" }
                p { class: "text-muted-foreground", "Profile screen coming soon..." }
            }
        }
    }
}
