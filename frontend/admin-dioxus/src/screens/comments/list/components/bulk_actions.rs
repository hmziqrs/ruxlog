use dioxus::prelude::*;
use ruxlog_shared::store::use_comments;
use oxui::shadcn::button::{Button, ButtonSize, ButtonVariant};

use super::super::context::use_comment_list_context;

#[component]
pub fn BulkActionsBar() -> Element {
    let ctx = use_comment_list_context();
    let comments = use_comments();
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
                    let selected_ids = ctx.selected_ids.read().clone();
                    rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let comments = comments;
                                let ids = selected_ids.clone();
                                spawn(async move {
                                    for id in ids {
                                        comments.unhide(id).await;
                                    }
                                });
                                ctx_clone.clear_selections();
                            },
                            "Unhide"
                        }
                    }
                }
                {
                    let mut ctx_clone = ctx.clone();
                    let selected_ids = ctx.selected_ids.read().clone();
                    rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let comments = comments;
                                let ids = selected_ids.clone();
                                spawn(async move {
                                    for id in ids {
                                        comments.hide(id).await;
                                    }
                                });
                                ctx_clone.clear_selections();
                            },
                            "Hide"
                        }
                    }
                }
                {
                    let mut ctx_clone = ctx.clone();
                    let selected_ids = ctx.selected_ids.read().clone();
                    rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            class: "text-red-600 border-red-200 dark:border-red-800",
                            onclick: move |_| {
                                let comments = comments;
                                let ids = selected_ids.clone();
                                spawn(async move {
                                    for id in ids {
                                        comments.delete_admin(id).await;
                                    }
                                });
                                ctx_clone.clear_selections();
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
