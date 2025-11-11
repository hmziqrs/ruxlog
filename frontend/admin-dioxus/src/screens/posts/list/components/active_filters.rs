use dioxus::prelude::*;

use crate::store::{use_categories, use_tag, use_user};
use crate::ui::shadcn::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant};

use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

use super::super::context::use_post_list_context;

#[component]
pub fn ActiveFilters(active_filter_count: usize) -> Element {
    if active_filter_count == 0 {
        return rsx! {};
    }

    let ctx = use_post_list_context();
    let categories_state = use_categories();
    let tags_state = use_tag();
    let users_state = use_user();

    // Get filter data
    let categories = categories_state
        .list
        .read()
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    let tags = tags_state
        .list
        .read()
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    let authors = users_state
        .list
        .read()
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "flex flex-wrap items-center gap-2",
            // Status filter badge
            if let Some(status) = ctx.filters.read().status.as_ref() {
                {
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Badge {
                            variant: BadgeVariant::Outline,
                            class: "gap-1.5",
                            "Status: {status}"
                            button {
                                class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                onclick: move |_| {
                                    ctx_clone.clear_status_filter();
                                },
                                Icon { icon: LdX {}, class: "w-3 h-3" }
                            }
                        }
                    }
                }
            }

            // Category filter badge
            for cat_id in ctx.selected_category_ids.read().iter() {
                if let Some(cat) = categories.iter().find(|c| c.id == *cat_id) {
                    {
                        let mut ctx_clone = ctx.clone();
                        rsx! {
                            Badge {
                                key: "cat-{cat_id}",
                                variant: BadgeVariant::Outline,
                                class: "gap-1.5",
                                "Category: {cat.name}"
                                button {
                                    class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                    onclick: move |_| {
                                        ctx_clone.clear_category_filter();
                                    },
                                    Icon { icon: LdX {}, class: "w-3 h-3" }
                                }
                            }
                        }
                    }
                }
            }

            // Tag filter badges
            for tag_id in ctx.selected_tag_ids.read().iter() {
                if let Some(tag) = tags.iter().find(|t| t.id == *tag_id) {
                    {
                        let tag_id_val = *tag_id;
                        let mut ctx_clone = ctx.clone();
                        rsx! {
                            Badge {
                                key: "tag-{tag_id}",
                                variant: BadgeVariant::Outline,
                                class: "gap-1.5",
                                "Tag: {tag.name}"
                                button {
                                    class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                    onclick: move |_| {
                                        ctx_clone.clear_tag_filter(tag_id_val);
                                    },
                                    Icon { icon: LdX {}, class: "w-3 h-3" }
                                }
                            }
                        }
                    }
                }
            }

            // Author filter badge
            for author_id in ctx.selected_author_ids.read().iter() {
                if let Some(author) = authors.iter().find(|a| a.id == *author_id) {
                    {
                        let mut ctx_clone = ctx.clone();
                        rsx! {
                            Badge {
                                key: "author-{author_id}",
                                variant: BadgeVariant::Outline,
                                class: "gap-1.5",
                                "Author: {author.name}"
                                button {
                                    class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                    onclick: move |_| {
                                        ctx_clone.clear_author_filter();
                                    },
                                    Icon { icon: LdX {}, class: "w-3 h-3" }
                                }
                            }
                        }
                    }
                }
            }

            // Clear all button
            {
                let mut ctx_clone = ctx.clone();
                rsx! {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "h-7 px-2",
                        onclick: move |_| { ctx_clone.clear_all_filters(); },
                        "Clear all"
                    }
                }
            }
        }
    }
}
