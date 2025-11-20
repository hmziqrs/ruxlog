use dioxus::prelude::*;

#[component]
pub fn ProfileEditScreen() -> Element {
    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8",
                h1 { class: "text-2xl font-bold mb-6", "Edit Profile" }
                p { class: "text-muted-foreground", "Profile edit screen coming soon..." }
            }
        }
    }
}
