use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Button, ButtonVariant};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

#[derive(Props, PartialEq, Clone)]
pub struct ConfirmDialogProps {
    pub is_open: Signal<bool>,
    pub title: String,
    pub description: String,
    #[props(default = "Confirm".to_string())]
    pub confirm_label: String,
    #[props(default = "Cancel".to_string())]
    pub cancel_label: String,
    #[props(default = "60".to_string())]
    pub z_index: String,
    pub on_confirm: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

/// A simple confirmation dialog that shows a title, description, and confirm/cancel buttons
#[component]
pub fn ConfirmDialog(mut props: ConfirmDialogProps) -> Element {
    if !*props.is_open.read() {
        return rsx! {};
    }

    rsx! {
        AppPortal {
            z_index: props.z_index.clone(),
            div {
                class: "fixed inset-0 bg-black/50",
                onclick: move |_| {
                    props.is_open.set(false);
                    props.on_cancel.call(());
                },
            }

            div {
                class: "fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%] w-full max-w-lg",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "bg-background rounded-lg border p-6 shadow-lg",
                    div { class: "flex items-start justify-between mb-4",
                        div {
                            h2 { class: "text-lg font-semibold", "{props.title}" }
                            p { class: "text-sm text-muted-foreground mt-1", "{props.description}" }
                        }
                        button {
                            onclick: move |_| {
                                props.is_open.set(false);
                                props.on_cancel.call(());
                            },
                            class: "rounded-xs opacity-70 hover:opacity-100 transition-opacity",
                            Icon { icon: LdX, width: 20, height: 20 }
                        }
                    }
                    div { class: "flex justify-end gap-2 mt-6",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                props.is_open.set(false);
                                props.on_cancel.call(());
                            },
                            "{props.cancel_label}"
                        }
                        Button {
                            onclick: move |_| {
                                props.is_open.set(false);
                                props.on_confirm.call(());
                            },
                            "{props.confirm_label}"
                        }
                    }
                }
            }
        }
    }
}
