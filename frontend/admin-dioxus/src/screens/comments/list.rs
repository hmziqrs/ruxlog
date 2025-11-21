use dioxus::prelude::*;
use ruxlog_shared::store::{use_comments, Comment, CommentListQuery};

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::containers::page_header::PageHeaderProps;
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::{Button, ButtonVariant};

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

    // Define header columns
    let headers = vec![
        HeaderColumn::new("ID", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Post", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("User", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Content", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Status", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Actions", false, "p-3 text-left font-medium text-xs md:text-sm", None),
    ];

    rsx! {
        DataTableScreen::<Comment> {
            frame: (comments.list)(),
            header: Some(PageHeaderProps {
                title: "Comments".to_string(),
                description: "Moderate comments and flags".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: reload,
                        "Refresh"
                    }
                }),
                class: None,
                embedded: false,
            }),
            headers: Some(headers),
            current_sort_field: None,
            on_sort: None,
            show_pagination: false,
            on_prev: move |_| {},
            on_next: move |_| {},
            below_toolbar: Some(rsx! {
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-3",
                    SimpleInput {
                        placeholder: Some("Filter by post id".to_string()),
                        value: post_id(),
                        oninput: move |value| post_id.set(value),
                        class: Some("text-sm".to_string()),
                    }
                    SimpleInput {
                        placeholder: Some("Filter by user id".to_string()),
                        value: user_id(),
                        oninput: move |value| user_id.set(value),
                        class: Some("text-sm".to_string()),
                    }
                    label { class: "flex items-center gap-2 text-sm",
                        input {
                            r#type: "checkbox",
                            checked: "{include_hidden()}",
                            onchange: move |_| include_hidden.toggle(),
                        }
                        "Include hidden"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: reload,
                        "Apply Filters"
                    }
                }
            }),
            if items.is_empty() {
                tr {
                    td { class: "p-4 text-center text-muted-foreground", colspan: "6",
                        "No comments found."
                    }
                }
            } else {
                {items.iter().cloned().map(|comment| {
                    rsx! {
                        CommentRow { key: "{comment.id}", comment }
                    }
                })}
            }
        }
    }
}

#[component]
fn CommentRow(comment: Comment) -> Element {
    let comments = use_comments();
    let hidden_label = if comment.hidden { "Hidden" } else { "Visible" };
    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "p-3", "{comment.id}" }
            td { class: "p-3", "{comment.post_id}" }
            td { class: "p-3", "{comment.user_id}" }
            td { class: "p-3 max-w-md truncate", "{comment.content}" }
            td { class: "p-3", "{hidden_label}" }
            td { class: "p-3 space-x-2",
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-8 px-2 text-xs",
                    onclick: move |_| {
                        let comments = comments;
                        let id = comment.id;
                        spawn(async move { comments.hide(id).await; });
                    },
                    "Hide"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-8 px-2 text-xs",
                    onclick: move |_| {
                        let comments = comments;
                        let id = comment.id;
                        spawn(async move { comments.unhide(id).await; });
                    },
                    "Unhide"
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    class: "h-8 px-2 text-xs",
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
