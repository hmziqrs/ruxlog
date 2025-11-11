use dioxus::prelude::*;

use crate::router::Route;
use crate::store::{use_post, ListStore, Post, PostStatus};
use crate::ui::shadcn::{
    Avatar, AvatarFallback, AvatarImage, Badge, BadgeVariant, Button, ButtonVariant, Card,
    CardContent, CardFooter, CardHeader, Checkbox, DropdownMenu, DropdownMenuContent,
    DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger,
};
use crate::utils::dates::format_short_date_dt;

use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCalendar, LdEllipsis, LdEye, LdHeart, LdMessageSquare, LdTag},
    Icon,
};

use super::super::context::use_post_list_context;
use super::super::utils::generate_avatar_fallback;

#[component]
pub fn GridView(posts: Vec<Post>, list_loading: bool, has_data: bool) -> Element {
    let nav = use_navigator();
    let posts_state = use_post();
    let ctx = use_post_list_context();

    rsx! {
        div { class: "min-h-[400px]",
            if posts.is_empty() {
                if list_loading && !has_data {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                        for i in 0..6 {
                            div {
                                key: "{i}",
                                class: "bg-white dark:bg-zinc-900 rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden animate-pulse",
                                div { class: "aspect-video bg-zinc-200 dark:bg-zinc-800" }
                                div { class: "p-4 space-y-3",
                                    div { class: "h-4 bg-zinc-200 dark:bg-zinc-800 rounded w-3/4" }
                                    div { class: "h-3 bg-zinc-200 dark:bg-zinc-800 rounded w-1/2" }
                                    div { class: "h-3 bg-zinc-200 dark:bg-zinc-800 rounded w-full" }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "flex flex-col items-center justify-center py-12 text-center",
                        Icon { icon: LdMessageSquare {}, class: "h-12 w-12 text-zinc-300 dark:text-zinc-700 mb-4" }
                        h3 { class: "text-lg font-medium", "No posts found" }
                        p { class: "text-zinc-500 dark:text-zinc-400 mt-1 max-w-md",
                            "Try adjusting your search or create a new post to get started."
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            class: "mt-4",
                            onclick: move |_| { nav.push(Route::PostsAddScreen {}); },
                            "Create Post"
                        }
                    }
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for post in posts.iter() {
                        {
                            let post_id = post.id;
                            let is_selected = ctx.selected_ids.read().contains(&post_id);
                            let mut ctx_clone = ctx.clone();

                            rsx! {
                                Card {
                                    key: "{post_id}",
                                    class: "overflow-hidden transition-all hover:shadow-lg dark:bg-zinc-900 group relative",
                                    div { class: "relative",
                                        if let Some(featured_image) = &post.featured_image {
                                            div { class: "aspect-video w-full overflow-hidden",
                                                img {
                                                    src: "{featured_image.file_url}",
                                                    alt: "{post.title}",
                                                    class: "h-full w-full object-cover transition-transform group-hover:scale-105",
                                                }
                                            }
                                        } else {
                                            div { class: "aspect-video w-full bg-zinc-200 dark:bg-zinc-800 flex items-center justify-center",
                                                Icon { icon: LdMessageSquare {}, class: "w-10 h-10 text-zinc-400 dark:text-zinc-600" }
                                            }
                                        }
                                        div { class: "absolute top-2 left-2",
                                            div { class: "bg-white dark:bg-zinc-800 rounded p-1 shadow-md",
                                                Checkbox {
                                                    checked: is_selected,
                                                    onchange: move |_| {
                                                        ctx_clone.toggle_post_selection(post_id);
                                                    },
                                                }
                                            }
                                        }
                                    }
                                    CardHeader { class: "p-4 pb-3",
                                        div { class: "flex items-start justify-between gap-2",
                                            div { class: "space-y-2 flex-1 min-w-0",
                                                div { class: "flex items-center gap-2 flex-wrap",
                                                    Badge {
                                                        variant: BadgeVariant::Outline,
                                                        class: "bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300",
                                                        "{post.category.name}"
                                                    }
                                                    {match post.status {
                                                        PostStatus::Published => rsx! {
                                                            Badge { variant: BadgeVariant::Secondary, class: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400", "Published" }
                                                        },
                                                        PostStatus::Draft => rsx! {
                                                            Badge { variant: BadgeVariant::Secondary, class: "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400", "Draft" }
                                                        },
                                                        PostStatus::Archived => rsx! {
                                                            Badge { variant: BadgeVariant::Secondary, class: "bg-zinc-100 text-zinc-800 dark:bg-zinc-800/30 dark:text-zinc-400", "Archived" }
                                                        },
                                                    }}
                                                }
                                                h3 { class: "font-semibold text-lg line-clamp-2", "{post.title}" }
                                            }
                                            DropdownMenu {
                                                DropdownMenuTrigger {
                                                    Button {
                                                        variant: ButtonVariant::Ghost,
                                                        class: "h-8 w-8 p-0 flex-shrink-0",
                                                        Icon { icon: LdEllipsis {}, class: "w-4 h-4" }
                                                    }
                                                }
                                                DropdownMenuContent {
                                                    DropdownMenuItem {
                                                        onclick: {
                                                            let nav = nav.clone();
                                                            move |_| { nav.push(Route::PostsViewScreen { id: post_id }); }
                                                        },
                                                        "View"
                                                    }
                                                    DropdownMenuItem {
                                                        onclick: {
                                                            let nav = nav.clone();
                                                            move |_| { nav.push(Route::PostsEditScreen { id: post_id }); }
                                                        },
                                                        "Edit"
                                                    }
                                                    DropdownMenuItem { "Duplicate" }
                                                    DropdownMenuSeparator {}
                                                    DropdownMenuItem {
                                                        class: "text-red-600 dark:text-red-400",
                                                        onclick: {
                                                            let posts_state = posts_state;
                                                            let filters = ctx.filters;
                                                            move |_| {
                                                                let posts_state = posts_state;
                                                                spawn(async move {
                                                                    posts_state.remove(post_id).await;
                                                                    let remove_state = posts_state.remove.read();
                                                                    if let Some(state) = remove_state.get(&post_id) {
                                                                        if state.is_success() {
                                                                            posts_state.fetch_list_with_query(filters()).await;
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        },
                                                        "Delete"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    CardContent { class: "px-4 pb-3",
                                        if let Some(excerpt) = &post.excerpt {
                                            p { class: "text-zinc-500 dark:text-zinc-400 text-sm line-clamp-2", "{excerpt}" }
                                        }
                                        if !post.tags.is_empty() {
                                            div { class: "flex flex-wrap gap-1.5 mt-3",
                                                for tag in post.tags.iter().take(3) {
                                                    Badge {
                                                        key: "{tag.id}",
                                                        variant: BadgeVariant::Secondary,
                                                        class: "text-xs",
                                                        "{tag.name}"
                                                    }
                                                }
                                                if post.tags.len() > 3 {
                                                    Badge { variant: BadgeVariant::Secondary, class: "text-xs", "+{post.tags.len() - 3}" }
                                                }
                                            }
                                        }
                                    }
                                    CardFooter { class: "px-4 pb-4 pt-0 flex-col gap-3 items-start",
                                        div { class: "w-full flex items-center justify-between",
                                            div { class: "flex items-center gap-2",
                                                Avatar { class: "w-7 h-7",
                                                    AvatarImage {
                                                        src: post.author.avatar.as_ref().map(|a| a.file_url.clone()).unwrap_or_default(),
                                                        alt: post.author.name.clone(),
                                                    }
                                                    AvatarFallback {
                                                        span { class: "text-xs font-medium", {generate_avatar_fallback(&post.author.name)} }
                                                    }
                                                }
                                                span { class: "text-xs text-zinc-500 dark:text-zinc-400 truncate max-w-[100px]", "{post.author.name}" }
                                            }
                                            div { class: "flex items-center gap-3 text-xs text-zinc-500 dark:text-zinc-400",
                                                div { class: "flex items-center gap-1",
                                                    Icon { icon: LdCalendar {}, class: "w-3.5 h-3.5" }
                                                    span {
                                                        if let Some(published_at) = &post.published_at {
                                                            {format_short_date_dt(published_at)}
                                                        } else {
                                                            {format_short_date_dt(&post.created_at)}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "w-full flex items-center gap-4 text-xs text-zinc-500 dark:text-zinc-400",
                                            div { class: "flex items-center gap-1",
                                                Icon { icon: LdEye {}, class: "w-3.5 h-3.5" }
                                                span { "{post.view_count}" }
                                            }
                                            div { class: "flex items-center gap-1",
                                                Icon { icon: LdHeart {}, class: "w-3.5 h-3.5" }
                                                span { "{post.likes_count}" }
                                            }
                                            div { class: "flex items-center gap-1",
                                                Icon { icon: LdTag {}, class: "w-3.5 h-3.5" }
                                                span { "{post.tags.len()}" }
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
}
