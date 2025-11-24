use dioxus::prelude::*;

use ruxlog_shared::store::SubscriberListQuery;
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonSize, ButtonVariant};

use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

use super::super::context::use_newsletter_list_context;
use super::super::utils::format_subscriber_status;

#[component]
pub fn ActiveFilters(active_filter_count: usize, filters: Signal<SubscriberListQuery>) -> Element {
    if active_filter_count == 0 {
        return rsx! {};
    }

    let ctx = use_newsletter_list_context();

    rsx! {
        div { class: "flex flex-wrap items-center gap-2",
            // Status filter badge
            if let Some(status) = filters.read().status {
                {
                    let mut ctx_clone = ctx.clone();
                    rsx! {
                        Badge {
                            variant: BadgeVariant::Outline,
                            class: "gap-1.5",
                            "Status: {format_subscriber_status(status)}"
                            button {
                                class: "ml-1 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-full",
                                onclick: {
                                    let mut filters = filters;
                                    move |_| {
                                        ctx_clone.clear_status_filter(&mut filters);
                                    }
                                },
                                Icon { icon: LdX {}, class: "w-3 h-3" }
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
                        onclick: {
                            let mut filters = filters;
                            move |_| { ctx_clone.clear_all_filters(&mut filters); }
                        },
                        "Clear all"
                    }
                }
            }
        }
    }
}
