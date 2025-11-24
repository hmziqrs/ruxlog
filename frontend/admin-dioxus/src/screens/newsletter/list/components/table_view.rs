use dioxus::prelude::*;

use crate::components::table::list_empty_state::ListEmptyState;
use crate::components::table::skeleton_table_rows::{
    SkeletonCellConfig, SkeletonTableRows, UICellType,
};
use ruxlog_shared::store::{use_newsletter, NewsletterSubscriber};
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

use super::super::context::use_newsletter_list_context;
use super::super::utils::{format_subscriber_status, status_badge_class};

#[component]
pub fn TableView(
    subscribers: Vec<NewsletterSubscriber>,
    list_loading: bool,
    has_data: bool,
    on_clear: EventHandler<()>,
) -> Element {
    let newsletter_state = use_newsletter();
    let ctx = use_newsletter_list_context();

    rsx! {
        if subscribers.is_empty() {
            if list_loading && !has_data {
                SkeletonTableRows {
                    row_count: 6,
                    cells: vec![
                        SkeletonCellConfig::custom(UICellType::Default, "w-12 py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Badge, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                        SkeletonCellConfig::custom(UICellType::Action, "w-12 py-2 px-3"),
                    ],
                }
            } else {
                tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                    td { colspan: "7", class: "py-12 px-4 text-center",
                        ListEmptyState {
                            title: "No subscribers found".to_string(),
                            description: "Try adjusting your search or filters.".to_string(),
                            clear_label: "Clear filters".to_string(),
                            create_label: "".to_string(),
                            on_clear: move |_| { on_clear.call(()); },
                            on_create: move |_| {},
                        }
                    }
                }
            }
        } else {
            for subscriber in subscribers.iter() {
                {
                    let subscriber_id = subscriber.id;
                    let is_selected = ctx.selected_ids.read().contains(&subscriber_id);
                    let mut ctx_clone = ctx.clone();

                    rsx! {
                        tr {
                            key: "{subscriber_id}",
                            class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                            td { class: "w-12 py-2 px-3",
                                Checkbox {
                                    checked: is_selected,
                                    onchange: move |_| {
                                        ctx_clone.toggle_subscriber_selection(subscriber_id);
                                    },
                                }
                            }
                            td { class: "py-2 px-3 text-sm", "{subscriber.id}" }
                            td { class: "py-2 px-3",
                                div { class: "font-medium text-sm", "{subscriber.email}" }
                            }
                            td { class: "py-2 px-3",
                                Badge {
                                    variant: BadgeVariant::Outline,
                                    class: "{status_badge_class(subscriber.status)}",
                                    "{format_subscriber_status(subscriber.status)}"
                                }
                            }
                            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap",
                                {format_short_date_dt(&subscriber.created_at)}
                            }
                            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap",
                                {format_short_date_dt(&subscriber.updated_at)}
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
                                        DropdownMenuItem { "View Details" }
                                        DropdownMenuItem { "Resend Confirmation" }
                                        DropdownMenuSeparator {}
                                        DropdownMenuItem {
                                            class: "text-red-600 dark:text-red-400",
                                            onclick: {
                                                let _newsletter_state = newsletter_state;
                                                let _email = subscriber.email.clone();
                                                move |_| {
                                                    // TODO: Implement unsubscribe action
                                                }
                                            },
                                            "Unsubscribe"
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
