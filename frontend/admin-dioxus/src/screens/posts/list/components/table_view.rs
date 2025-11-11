use dioxus::prelude::*;

use crate::components::{ListEmptyState, SkeletonCellConfig, SkeletonTableRows, UICellType};
use crate::router::Route;
use crate::store::{use_post, ListStore, Post, PostStatus};
use crate::ui::shadcn::{
    Avatar, AvatarFallback, AvatarImage, Badge, BadgeVariant, Button, ButtonVariant, Checkbox,
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator,
    DropdownMenuTrigger,
};
use crate::utils::dates::format_short_date_dt;

use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdEllipsis, LdEye, LdHeart, LdMessageSquare, LdTag},
    Icon,
};

use super::super::context::use_post_list_context;
use super::super::utils::generate_avatar_fallback;

#[component]
pub fn TableView(
    posts: Vec<Post>,
    list_loading: bool,
    has_data: bool,
    on_clear: EventHandler<()>,
) -> Element {
    let nav = use_navigator();
    let posts_state = use_post();
    let ctx = use_post_list_context();

    rsx! {
        if posts.is_empty() {
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
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Action, "w-12 py-2 px-3"),
                    ],
                }
            } else {
                tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                    td { colspan: "10", class: "py-12 px-4 text-center",
                        ListEmptyState {
                            title: "No posts found".to_string(),
                            description: "Try adjusting your search or create a new post to get started.".to_string(),
                            clear_label: "Clear search".to_string(),
                            create_label: "Create your first post".to_string(),
                            on_clear: move |_| { on_clear.call(()); },
                            on_create: move |_| { nav.push(Route::PostsAddScreen {}); },
                        }
                    }
                }
            }
        } else {
            for post in posts.iter() {
                {
                    let post_id = post.id;
                    let is_selected = ctx.selected_ids.read().contains(&post_id);
                    let mut ctx_clone = ctx.clone();

                    rsx! {
                        tr {
                            key: "{post_id}",
                            class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                            td { class: "w-12 py-2 px-3",
                                Checkbox {
                                    checked: is_selected,
                                    onchange: move |_| {
                                        ctx_clone.toggle_post_selection(post_id);
                                    },
                                }
                            }
                            td { class: "py-2 px-3",
                                div { class: "flex items-center gap-3",
                                    if let Some(featured_image) = &post.featured_image {
                                        div { class: "w-12 h-12 rounded overflow-hidden flex-shrink-0",
                                            img {
                                                src: "{featured_image.file_url}",
                                                alt: "{post.title}",
                                                class: "w-full h-full object-cover",
                                            }
                                        }
                                    } else {
                                        div { class: "w-12 h-12 rounded bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center flex-shrink-0",
                                            Icon { icon: LdMessageSquare {}, class: "w-5 h-5 text-zinc-400 dark:text-zinc-600" }
                                        }
                                    }
                                    div { class: "min-w-0",
                                        div { class: "font-medium text-sm truncate max-w-xs", "{post.title}" }
                                        if let Some(excerpt) = &post.excerpt {
                                            p { class: "text-xs text-muted-foreground truncate max-w-xs mt-0.5", "{excerpt}" }
                                        }
                                    }
                                }
                            }
                            td { class: "py-2 px-3",
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
                                    span { class: "text-xs font-medium text-zinc-700 dark:text-zinc-300 truncate max-w-[120px]", "{post.author.name}" }
                                }
                            }
                            td { class: "py-2 px-3",
                                Badge {
                                    variant: BadgeVariant::Outline,
                                    class: "bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300",
                                    "{post.category.name}"
                                }
                            }
                            td { class: "py-2 px-3",
                                {match post.status {
                                    PostStatus::Published => rsx! {
                                        Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Published" }
                                    },
                                    PostStatus::Draft => rsx! {
                                        Badge { variant: BadgeVariant::Secondary, class: "bg-amber-100 text-amber-800 border-amber-200 dark:bg-amber-900/20 dark:text-amber-400", "Draft" }
                                    },
                                    PostStatus::Archived => rsx! {
                                        Badge { variant: BadgeVariant::Secondary, class: "bg-zinc-100 text-zinc-800 border-zinc-200 dark:bg-zinc-800/30 dark:text-zinc-400", "Archived" }
                                    },
                                }}
                            }
                            td { class: "py-2 px-3",
                                div { class: "flex items-center gap-3 text-xs text-muted-foreground",
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
                            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap",
                                {if let Some(published_at) = &post.published_at {
                                    format_short_date_dt(published_at)
                                } else {
                                    "—".to_string()
                                }}
                            }
                            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap",
                                {format_short_date_dt(&post.created_at)}
                            }
                            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap",
                                {format_short_date_dt(&post.updated_at)}
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
                    }
                }
            }
        }
    }
}
