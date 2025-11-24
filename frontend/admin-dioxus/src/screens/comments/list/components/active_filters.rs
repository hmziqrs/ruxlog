use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};
use oxstore::ListQuery;
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonSize, ButtonVariant};
use ruxlog_shared::store::{use_post, use_user, CommentListQuery, FlagFilter, HiddenFilter};

use super::super::context::use_comment_list_context;

#[component]
pub fn ActiveFilters(active_filter_count: usize, filters: Signal<CommentListQuery>) -> Element {
    if active_filter_count == 0 {
        return rsx! {};
    }

    let ctx = use_comment_list_context();
    let posts_state = use_post();
    let users_state = use_user();

    let posts = posts_state
        .list
        .read()
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    let users = users_state
        .list
        .read()
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    let selected_user_id = *ctx.selected_user_id.read();
    let selected_post_id = *ctx.selected_post_id.read();
    let selected_flag_filter = ctx.selected_flag_filter.read().clone();
    let hidden_filter = filters.read().hidden_filter.clone();

    rsx! {
        div { class: "flex flex-wrap items-center gap-2",
            if let Some(user_id) = selected_user_id {
                {
                    let user_label = users
                        .iter()
                        .find(|u| u.id == user_id)
                        .map(|u| u.name.clone())
                        .unwrap_or_else(|| format!("User #{}", user_id));
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Badge {
                            variant: BadgeVariant::Outline,
                            class: "gap-1.5",
                            "User: {user_label}"
                            button {
                                class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                onclick: {
                                    let mut filters = filters;
                                    move |_| {
                                        ctx_clone.clear_user_filter(&mut filters);
                                    }
                                },
                                Icon { icon: LdX {}, class: "w-3 h-3" }
                            }
                        }
                    }
                }
            }

            if let Some(post_id) = selected_post_id {
                {
                    let post_label = posts
                        .iter()
                        .find(|p| p.id == post_id)
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| format!("Post #{}", post_id));
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Badge {
                            variant: BadgeVariant::Outline,
                            class: "gap-1.5",
                            "Post: {post_label}"
                            button {
                                class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                onclick: {
                                    let mut filters = filters;
                                    move |_| {
                                        ctx_clone.clear_post_filter(&mut filters);
                                    }
                                },
                                Icon { icon: LdX {}, class: "w-3 h-3" }
                            }
                        }
                    }
                }
            }

            if selected_flag_filter != FlagFilter::All {
                {
                    let flag_label = describe_flag_filter(selected_flag_filter);
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Badge {
                            variant: BadgeVariant::Outline,
                            class: "gap-1.5",
                            "Flags: {flag_label}"
                            button {
                                class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                onclick: {
                                    let mut filters = filters;
                                    move |_| {
                                        ctx_clone.set_flag_filter(&mut filters, FlagFilter::All);
                                    }
                                },
                                Icon { icon: LdX {}, class: "w-3 h-3" }
                            }
                        }
                    }
                }
            }

            if let Some(hidden) =
                hidden_filter.filter(|h| matches!(h, HiddenFilter::Hidden | HiddenFilter::All))
            {
                {
                    let status_label = describe_hidden_filter(hidden);
                    rsx! {
                        Badge {
                            variant: BadgeVariant::Outline,
                            class: "gap-1.5",
                            "Status: {status_label}"
                            button {
                                class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                onclick: {
                                    let mut filters = filters;
                                    move |_| {
                                        let mut query = filters.peek().clone();
                                        query.set_page(1);
                                        query.hidden_filter = None;
                                        filters.set(query);
                                    }
                                },
                                Icon { icon: LdX {}, class: "w-3 h-3" }
                            }
                        }
                    }
                }
            }

            {
                let mut ctx_clone = ctx.clone();
                rsx! {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "h-7 px-2",
                        onclick: {
                            let mut filters = filters;
                            move |_| {
                                ctx_clone.clear_all_filters(&mut filters);
                            }
                        },
                        "Clear all"
                    }
                }
            }
        }
    }
}

fn describe_flag_filter(filter: FlagFilter) -> &'static str {
    match filter {
        FlagFilter::All => "All",
        FlagFilter::Flagged => "Flagged",
        FlagFilter::NotFlagged => "Not flagged",
    }
}

fn describe_hidden_filter(filter: HiddenFilter) -> &'static str {
    match filter {
        HiddenFilter::Visible => "Visible",
        HiddenFilter::Hidden => "Hidden",
        HiddenFilter::All => "All",
    }
}
