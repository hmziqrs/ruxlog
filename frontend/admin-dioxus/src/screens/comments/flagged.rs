use dioxus::prelude::*;
use ruxlog_shared::store::{use_comments, CommentFlag};

#[component]
pub fn FlaggedCommentsScreen() -> Element {
    let comments = use_comments();

    use_effect(move || {
        spawn(async move {
            comments.list_flags().await;
        });
    });

    let refresh = move |_| {
        let comments = comments;
        spawn(async move { comments.list_flags().await; });
    };

    let flags = comments
        .flags
        .read()
        .data
        .clone()
        .unwrap_or_default();

    rsx! {
        div { class: "p-6 space-y-4",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-2xl font-semibold", "Flagged Comments" }
                    p { class: "text-sm text-muted-foreground", "Review and clear comment flags." }
                }
                button {
                    class: "inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-accent",
                    onclick: refresh,
                    "Refresh"
                }
            }

            div { class: "overflow-auto bg-transparent border border-border rounded-lg",
                table { class: "w-full text-sm",
                    thead { class: "bg-transparent",
                        tr {
                            th { class: "p-3 text-left", "Flag ID" }
                            th { class: "p-3 text-left", "Comment" }
                            th { class: "p-3 text-left", "User" }
                            th { class: "p-3 text-left", "Reason" }
                            th { class: "p-3 text-left", "Actions" }
                        }
                    }
                    tbody {
                        if flags.is_empty() {
                            tr {
                                td { class: "p-4 text-center text-muted-foreground", colspan: "5",
                                    "No flags found."
                                }
                            }
                        } else {
                            for flag in flags {
                                FlagRow { flag }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FlagRow(flag: CommentFlag) -> Element {
    let comments = use_comments();
    rsx! {
        tr { class: "border-t border-border",
            td { class: "p-3", "{flag.id}" }
            td { class: "p-3", "{flag.comment_id}" }
            td { class: "p-3", "{flag.user_id}" }
            td { class: "p-3 max-w-md truncate", "{flag.reason}" }
            td { class: "p-3 space-x-2",
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs hover:bg-accent",
                    onclick: move |_| {
                        let comments = comments;
                        let id = flag.comment_id;
                        spawn(async move { comments.clear_flags(id).await; });
                    },
                    "Clear flags"
                }
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs text-red-600 hover:bg-red-50",
                    onclick: move |_| {
                        let comments = comments;
                        let id = flag.comment_id;
                        spawn(async move { comments.delete_admin(id).await; });
                    },
                    "Delete comment"
                }
            }
        }
    }
}
