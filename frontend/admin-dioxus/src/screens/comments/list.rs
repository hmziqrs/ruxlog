use dioxus::prelude::*;
use ruxlog_shared::store::{use_comments, Comment, CommentListQuery};

#[component]
pub fn CommentsListScreen() -> Element {
    let comments = use_comments();
    let mut post_id = use_signal(|| "".to_string());
    let mut user_id = use_signal(|| "".to_string());
    let mut include_hidden = use_signal(|| true);

    // Initial load
    use_effect(move || {
        spawn({
            let comments = comments;
            async move {
                comments
                    .admin_list(CommentListQuery {
                        post_id: None,
                        user_id: None,
                        page: 1,
                        limit: Some(50),
                        include_hidden: Some(true),
                        ..Default::default()
                    })
                    .await;
            }
        });
    });

    let reload = move |_| {
        let comments = comments;
        let post_filter = post_id().parse::<i32>().ok();
        let user_filter = user_id().parse::<i32>().ok();
        let include_hidden = include_hidden();
        spawn(async move {
            comments
                .admin_list(CommentListQuery {
                    post_id: post_filter,
                    user_id: user_filter,
                    page: 1,
                    limit: Some(50),
                    include_hidden: Some(include_hidden),
                    ..Default::default()
                })
                .await;
        });
    };

    let list = comments.list.read();
    let items: Vec<Comment> = list
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "p-6 space-y-4",
            div { class: "flex items-center justify-between",
                div { class: "space-y-1",
                    h2 { class: "text-2xl font-semibold", "Comments" }
                    p { class: "text-sm text-muted-foreground", "Moderate comments and flags." }
                }
                button {
                    class: "inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-accent",
                    onclick: reload,
                    "Refresh"
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-3",
                input {
                    class: "w-full rounded-md border border-border px-3 py-2 text-sm",
                    placeholder: "Filter by post id",
                    value: "{post_id}",
                    oninput: move |e| post_id.set(e.value())
                }
                input {
                    class: "w-full rounded-md border border-border px-3 py-2 text-sm",
                    placeholder: "Filter by user id",
                    value: "{user_id}",
                    oninput: move |e| user_id.set(e.value())
                }
                label { class: "flex items-center gap-2 text-sm",
                    input {
                        r#type: "checkbox",
                        checked: "{include_hidden()}",
                        onchange: move |_| include_hidden.toggle(),
                    }
                    "Include hidden"
                }
                button {
                    class: "rounded-md border border-border px-3 py-2 text-sm font-medium hover:bg-accent",
                    onclick: reload,
                    "Apply Filters"
                }
            }

            div { class: "overflow-auto bg-transparent border border-border rounded-lg",
                table { class: "w-full text-sm",
                    thead { class: "bg-transparent",
                        tr {
                            th { class: "p-3 text-left", "ID" }
                            th { class: "p-3 text-left", "Post" }
                            th { class: "p-3 text-left", "User" }
                            th { class: "p-3 text-left", "Content" }
                            th { class: "p-3 text-left", "Status" }
                            th { class: "p-3 text-left", "Actions" }
                        }
                    }
                    tbody {
                        if items.is_empty() {
                            tr {
                                td { class: "p-4 text-center text-muted-foreground", colspan: "6",
                                    "No comments found."
                                }
                            }
                        } else {
                            for comment in items {
                        CommentRow { key: "{comment.id}", comment }
                    }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommentRow(comment: Comment) -> Element {
    let comments = use_comments();
    let hidden_label = if comment.is_hidden { "Hidden" } else { "Visible" };
    rsx! {
        tr { class: "border-t border-border",
            td { class: "p-3", "{comment.id}" }
            td { class: "p-3", "{comment.post_id}" }
            td { class: "p-3", "{comment.user_id}" }
            td { class: "p-3 max-w-md truncate", "{comment.content}" }
            td { class: "p-3", "{hidden_label}" }
            td { class: "p-3 space-x-2",
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs hover:bg-accent",
                    onclick: move |_| {
                        let comments = comments;
                        let id = comment.id;
                        spawn(async move { comments.hide(id).await; });
                    },
                    "Hide"
                }
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs hover:bg-accent",
                    onclick: move |_| {
                        let comments = comments;
                        let id = comment.id;
                        spawn(async move { comments.unhide(id).await; });
                    },
                    "Unhide"
                }
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs text-red-600 hover:bg-red-50",
                    onclick: move |_| {
                        let comments = comments;
                        let id = comment.id;
                        spawn(async move { comments.delete_admin(id).await; });
                    },
                    "Delete"
                }
            }
        }
    }
}
