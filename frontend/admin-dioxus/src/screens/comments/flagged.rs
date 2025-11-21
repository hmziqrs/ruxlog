use dioxus::prelude::*;
use ruxlog_shared::store::{use_comments, CommentFlag};
use oxstore::{PaginatedList, StateFrame};

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::containers::page_header::PageHeaderProps;
use oxui::shadcn::button::{Button, ButtonVariant};

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
    let flags_frame = to_paginated_frame((comments.flags)());

    // Define header columns
    let headers = vec![
        HeaderColumn::new("Flag ID", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Comment", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("User", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Reason", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Actions", false, "p-3 text-left font-medium text-xs md:text-sm", None),
    ];

    rsx! {
        DataTableScreen::<CommentFlag> {
            frame: flags_frame,
            header: Some(PageHeaderProps {
                title: "Flagged Comments".to_string(),
                description: "Review and clear comment flags".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: refresh,
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
            if flags.is_empty() {
                tr {
                    td { class: "p-4 text-center text-muted-foreground", colspan: "5",
                        "No flags found."
                    }
                }
            } else {
                {flags.iter().cloned().map(|flag| {
                    rsx! {
                        FlagRow { key: "{flag.id}", flag }
                    }
                })}
            }
        }
    }
}

fn to_paginated_frame<T: Clone>(frame: StateFrame<Vec<T>>) -> StateFrame<PaginatedList<T>> {
    let data_vec = frame.data;
    let count = data_vec.as_ref().map(|d| d.len() as u64).unwrap_or(0);
    let data = data_vec.map(|items| PaginatedList {
        data: items,
        total: count,
        page: 1,
        per_page: count.max(1),
    });

    StateFrame {
        status: frame.status,
        data,
        meta: frame.meta,
        error: frame.error,
    }
}

#[component]
fn FlagRow(flag: CommentFlag) -> Element {
    let comments = use_comments();
    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "p-3", "{flag.id}" }
            td { class: "p-3", "{flag.comment_id}" }
            td { class: "p-3", "{flag.user_id}" }
            td { class: "p-3 max-w-md truncate", "{flag.reason}" }
            td { class: "p-3 space-x-2",
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-8 px-2 text-xs",
                    onclick: move |_| {
                        let comments = comments;
                        let id = flag.comment_id;
                        spawn(async move { comments.clear_flags(id).await; });
                    },
                    "Clear flags"
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    class: "h-8 px-2 text-xs",
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
