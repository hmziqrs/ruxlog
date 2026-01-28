use dioxus::prelude::*;

#[component]
pub fn BannerPlaceholder() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 max-w-6xl py-8",
            div { class: "w-full h-32 bg-muted rounded-lg flex items-center justify-center",
                span { class: "text-muted-foreground text-sm", "Advertisement / Banner Placeholder" }
            }
        }
    }
}
