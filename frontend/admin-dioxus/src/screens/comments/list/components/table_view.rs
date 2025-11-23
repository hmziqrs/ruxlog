use dioxus::prelude::*;

use crate::components::table::list_empty_state::ListEmptyState;
use crate::components::table::skeleton_table_rows::{
    SkeletonCellConfig, SkeletonTableRows, UICellType,
};
use ruxlog_shared::store::{use_comments, Comment};
use oxui::shadcn::avatar::{Avatar, AvatarFallback};
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::checkbox::Checkbox;
use oxui::shadcn::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger,
};
use crate::utils::dates::format_short_date_dt;

use hmziq_dioxus_free_icons::{
    icons::ld_icons::LdEllipsis,
    Icon,
};

use super::super::context::use_comment_list_context;

fn generate_avatar_fallback(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn TableView(
    comments: Vec<Comment>,
    list_loading: bool,
    has_data: bool,
    on_clear: EventHandler<()>,
) -> Element {
    let comments_state = use_comments();
    let ctx = use_comment_list_context();

    rsx! {
        if comments.is_empty() {
            if list_loading && !has_data {
                SkeletonTableRows {
                    row_count: 6,
                    cells: vec![
                        SkeletonCellConfig::custom(UICellType::Default, "w-12 py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Badge, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Action, "w-12 py-2 px-3"),
                    ],
                }
            } else {
                tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                    td { colspan: "7", class: "py-12 px-4 text-center",
                        ListEmptyState {
                            title: "No comments found".to_string(),
                            description: "Try adjusting your search or filters.".to_string(),
                            clear_label: "Clear search".to_string(),
                            create_label: "".to_string(),
                            on_clear: move |_| { on_clear.call(()); },
                            on_create: move |_| {},
                        }
                    }
                }
            }
        } else {
            for comment in comments.iter() {
                {
                    let comment_id = comment.id;
                    let is_selected = ctx.selected_ids.read().contains(&comment_id);
                    let is_hidden = comment.hidden;
                    let mut ctx_clone = ctx.clone();

                    rsx! {
                        tr {
                            key: "{comment_id}",
                            class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                            td { class: "w-12 py-2 px-3",
                                Checkbox {
                                    checked: is_selected,
                                    onchange: move |_| {
                                        ctx_clone.toggle_comment_selection(comment_id);
                                    },
                                }
                            }
                            td { class: "py-2 px-3 text-sm text-zinc-700 dark:text-zinc-300",
                                "Post #{comment.post_id}"
                            }
                            td { class: "py-2 px-3",
                                div { class: "flex items-center gap-2",
                                    if let Some(author) = &comment.author {
                                        Avatar { class: "w-7 h-7",
                                            AvatarFallback {
                                                span { class: "text-xs font-medium", {generate_avatar_fallback(&author.name)} }
                                            }
                                        }
                                        span { class: "text-xs font-medium text-zinc-700 dark:text-zinc-300 truncate max-w-[120px]", "{author.name}" }
                                    } else {
                                        span { class: "text-xs text-muted-foreground", "User #{comment.user_id}" }
                                    }
                                }
                            }
                            td { class: "py-2 px-3 max-w-md",
                                p { class: "text-sm text-zinc-700 dark:text-zinc-300 truncate", "{comment.content}" }
                            }
                            td { class: "py-2 px-3",
                                {if is_hidden {
                                    rsx! {
                                        Badge { variant: BadgeVariant::Secondary, class: "bg-red-100 text-red-800 border-red-200 dark:bg-red-900/20 dark:text-red-400", "Hidden" }
                                    }
                                } else {
                                    rsx! {
                                        Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Visible" }
                                    }
                                }}
                            }
                            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap",
                                {format_short_date_dt(&comment.created_at)}
                            }
                            td { class: "w-12 py-2 px-3",
                                DropdownMenu {
                                    DropdownMenuTrigger { class: "h-8 w-8",
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            class: "h-8 w-8 p-0",
                                            Icon { icon: LdEllipsis {}, class: "w-4 h-4" }
                                        }
                                    }
                                    DropdownMenuContent { class: "w-40",
                                        DropdownMenuItem {
                                            onclick: move |_| {
                                                // TODO: Navigate to view comment
                                            },
                                            "View"
                                        }
                                        DropdownMenuItem {
                                            onclick: {
                                                let comments_state = comments_state;
                                                move |_| {
                                                    let comments_state = comments_state;
                                                    spawn(async move {
                                                        if is_hidden {
                                                            comments_state.unhide(comment_id).await;
                                                        } else {
                                                            comments_state.hide(comment_id).await;
                                                        }
                                                    });
                                                }
                                            },
                                            if is_hidden { "Unhide" } else { "Hide" }
                                        }
                                        DropdownMenuSeparator {}
                                        DropdownMenuItem {
                                            class: "text-red-600 dark:text-red-400",
                                            onclick: {
                                                let comments_state = comments_state;
                                                move |_| {
                                                    let comments_state = comments_state;
                                                    spawn(async move {
                                                        comments_state.delete_admin(comment_id).await;
                                                    });
                                                }
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
