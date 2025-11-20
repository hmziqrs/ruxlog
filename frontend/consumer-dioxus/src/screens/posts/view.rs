use dioxus::prelude::*;

#[component]
pub fn PostViewScreen(id: i32) -> Element {
    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8",
                "Post View Screen for post {id}"
            }
        }
    }
}
