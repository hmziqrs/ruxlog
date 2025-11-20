use dioxus::prelude::*;

#[component]
pub fn LoginScreen() -> Element {
    rsx! {
        div { class: "min-h-screen bg-background text-foreground flex items-center justify-center",
            div { class: "w-full max-w-md p-8",
                h1 { class: "text-2xl font-bold mb-6", "Login" }
                p { class: "text-muted-foreground", "Login form coming soon..." }
            }
        }
    }
}
