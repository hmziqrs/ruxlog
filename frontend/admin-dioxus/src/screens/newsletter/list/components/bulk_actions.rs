use dioxus::prelude::*;

use oxui::shadcn::button::{Button, ButtonSize, ButtonVariant};

use super::super::context::use_newsletter_list_context;

#[component]
pub fn BulkActionsBar() -> Element {
    let ctx = use_newsletter_list_context();
    let selected_count = ctx.selected_ids.read().len();

    if selected_count == 0 {
        return rsx! {};
    }

    rsx! {
        div { class: "w-full flex items-center justify-between bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-md px-4 py-3 shadow-sm",
            span { class: "text-sm text-zinc-500 dark:text-zinc-400",
                "{selected_count} selected"
            }
            div { class: "flex items-center gap-2",
                {
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                // TODO: Implement bulk export
                                ctx_clone.clear_selections();
                            },
                            "Export"
                        }
                    }
                }
                {
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            class: "text-red-600 border-red-200 dark:border-red-800",
                            onclick: move |_| {
                                // TODO: Implement bulk unsubscribe
                                ctx_clone.clear_selections();
                            },
                            "Unsubscribe"
                        }
                    }
                }
            }
        }
    }
}
