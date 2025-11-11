use dioxus::prelude::*;

use crate::{
    components::{ErrorDetails, ErrorDetailsVariant},
    store::use_auth,
    ui::shadcn::{Button, ButtonVariant},
};

#[component]
pub fn AuthGuardError(on_retry: EventHandler<()>) -> Element {
    let auth_store = use_auth();
    let init_status = auth_store.init_status.read();

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-background p-4",
            div { class: "max-w-md w-full",
                div { class: "rounded-xl border border-border/60 bg-background p-8 shadow-lg space-y-6",
                    div { class: "flex justify-center mb-2",
                        img {
                            class: "h-24 w-24",
                            src: asset!("/assets/logo.png"),
                            alt: "Logo",
                        }
                    }
                    ErrorDetails {
                        error: init_status.error.clone(),
                        variant: ErrorDetailsVariant::Minimum,
                        class: Some("w-full".to_string()),
                    }
                    div { class: "flex justify-center pt-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| on_retry.call(()),
                            "Try Again"
                        }
                    }
                }
            }
        }
    }
}
