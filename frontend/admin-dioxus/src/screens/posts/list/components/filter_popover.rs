use dioxus::prelude::*;

use crate::store::{use_categories, use_tag, use_user};
use crate::ui::shadcn::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Checkbox, Popover, PopoverContent,
    PopoverTrigger,
};

use hmziq_dioxus_free_icons::{icons::ld_icons::LdFilter, Icon};

use super::super::context::use_post_list_context;

#[component]
pub fn FilterPopover(active_filter_count: usize) -> Element {
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
        Popover {
            PopoverTrigger {
                Button {
                    variant: ButtonVariant::Outline,
                    class: "gap-2",
                    Icon { icon: LdFilter {}, class: "h-4 w-4" }
                    "Filters"
                    if active_filter_count > 0 {
                        Badge {
                            variant: BadgeVariant::Secondary,
                            class: "ml-1 px-1.5 min-w-5 h-5 rounded-full",
                            "{active_filter_count}"
                        }
                    }
                }
            }
            PopoverContent { class: "w-80 p-4",
                div { class: "space-y-4",
                    div { class: "flex items-center justify-between",
                        h4 { class: "font-semibold text-sm", "Filters" }
                        if active_filter_count > 0 {
                            {
                                let mut ctx_clone = ctx.clone();
                                rsx! {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        size: ButtonSize::Sm,
                                        class: "h-auto p-0 text-xs",
                                        onclick: move |_| { ctx_clone.clear_all_filters(); },
                                        "Clear all"
                                    }
                                }
                            }
                        }
                    }

                    if !categories.is_empty() {
                        div { class: "space-y-2",
                            h5 { class: "text-sm font-medium", "Categories" }
                            div { class: "max-h-40 overflow-y-auto space-y-2 pr-2",
                                for cat in categories.iter() {
                                    {
                                        let cat_id = cat.id;
                                        let is_selected = ctx.selected_category_ids.read().contains(&cat_id);
                                        let mut ctx_clone = ctx.clone();
                                        rsx! {
                                            div {
                                                key: "{cat_id}",
                                                class: "flex items-center gap-2",
                                                Checkbox {
                                                    checked: is_selected,
                                                    onchange: move |_| {
                                                        ctx_clone.toggle_category(cat_id);
                                                    },
                                                }
                                                label { class: "text-sm cursor-pointer", "{cat.name}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "-mx-4 my-2 h-px bg-zinc-200 dark:bg-zinc-800" }
                    }

                    if !tags.is_empty() {
                        div { class: "space-y-2",
                            h5 { class: "text-sm font-medium", "Tags" }
                            div { class: "max-h-40 overflow-y-auto space-y-2 pr-2",
                                for tag in tags.iter() {
                                    {
                                        let tag_id = tag.id;
                                        let is_selected = ctx.selected_tag_ids.read().contains(&tag_id);
                                        let mut ctx_clone = ctx.clone();
                                        rsx! {
                                            div {
                                                key: "{tag_id}",
                                                class: "flex items-center gap-2",
                                                Checkbox {
                                                    checked: is_selected,
                                                    onchange: move |_| {
                                                        ctx_clone.toggle_tag(tag_id);
                                                    },
                                                }
                                                label { class: "text-sm cursor-pointer", "{tag.name}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "-mx-4 my-2 h-px bg-zinc-200 dark:bg-zinc-800" }
                    }

                    if !authors.is_empty() {
                        div { class: "space-y-2",
                            h5 { class: "text-sm font-medium", "Authors" }
                            div { class: "max-h-40 overflow-y-auto space-y-2 pr-2",
                                for author in authors.iter() {
                                    {
                                        let author_id = author.id;
                                        let is_selected = ctx.selected_author_ids.read().contains(&author_id);
                                        let mut ctx_clone = ctx.clone();
                                        rsx! {
                                            div {
                                                key: "{author_id}",
                                                class: "flex items-center gap-2",
                                                Checkbox {
                                                    checked: is_selected,
                                                    onchange: move |_| {
                                                        ctx_clone.toggle_author(author_id);
                                                    },
                                                }
                                                label { class: "text-sm cursor-pointer", "{author.name}" }
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
