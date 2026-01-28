use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn BannerPlaceholder() -> Element {
    let nav = use_navigator();

    rsx! {
        div { class: "container mx-auto px-4 max-w-6xl py-8",
            button {
                class: "w-full h-32 bg-muted rounded-lg flex items-center justify-center hover:bg-muted/80 transition-colors cursor-pointer",
                onclick: move |_| {
                    nav.push(Route::ContactScreen {});
                },
                span { class: "text-muted-foreground text-sm", "Advertise here" }
            }
        }
    }
}
