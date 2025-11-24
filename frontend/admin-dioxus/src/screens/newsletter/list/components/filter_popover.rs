use dioxus::prelude::*;

use ruxlog_shared::store::{SubscriberListQuery, SubscriberStatus};
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonSize, ButtonVariant};
use oxui::shadcn::popover::{Popover, PopoverContent, PopoverTrigger};

use hmziq_dioxus_free_icons::{icons::ld_icons::LdFilter, Icon};

use super::super::context::use_newsletter_list_context;

#[component]
pub fn FilterPopover(active_filter_count: usize, filters: Signal<SubscriberListQuery>) -> Element {
    let ctx = use_newsletter_list_context();

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

                    div { class: "space-y-2",
                        h5 { class: "text-sm font-medium", "Status" }
                        div { class: "space-y-2",
                            {
                                let mut ctx_clone = ctx.clone();
                                let is_selected = ctx.selected_status.read().is_none();
                                rsx! {
                                    Button {
                                        variant: if is_selected { ButtonVariant::Default } else { ButtonVariant::Outline },
                                        size: ButtonSize::Sm,
                                        class: "w-full justify-start",
                                        onclick: {
                                            let mut filters = filters;
                                            move |_| {
                                                ctx_clone.set_status(&mut filters, None);
                                            }
                                        },
                                        "All"
                                    }
                                }
                            }
                            {
                                let mut ctx_clone = ctx.clone();
                                let is_selected = *ctx.selected_status.read() == Some(SubscriberStatus::Confirmed);
                                rsx! {
                                    Button {
                                        variant: if is_selected { ButtonVariant::Default } else { ButtonVariant::Outline },
                                        size: ButtonSize::Sm,
                                        class: "w-full justify-start",
                                        onclick: {
                                            let mut filters = filters;
                                            move |_| {
                                                ctx_clone.set_status(&mut filters, Some(SubscriberStatus::Confirmed));
                                            }
                                        },
                                        "Confirmed"
                                    }
                                }
                            }
                            {
                                let mut ctx_clone = ctx.clone();
                                let is_selected = *ctx.selected_status.read() == Some(SubscriberStatus::Pending);
                                rsx! {
                                    Button {
                                        variant: if is_selected { ButtonVariant::Default } else { ButtonVariant::Outline },
                                        size: ButtonSize::Sm,
                                        class: "w-full justify-start",
                                        onclick: {
                                            let mut filters = filters;
                                            move |_| {
                                                ctx_clone.set_status(&mut filters, Some(SubscriberStatus::Pending));
                                            }
                                        },
                                        "Pending"
                                    }
                                }
                            }
                            {
                                let mut ctx_clone = ctx.clone();
                                let is_selected = *ctx.selected_status.read() == Some(SubscriberStatus::Unsubscribed);
                                rsx! {
                                    Button {
                                        variant: if is_selected { ButtonVariant::Default } else { ButtonVariant::Outline },
                                        size: ButtonSize::Sm,
                                        class: "w-full justify-start",
                                        onclick: {
                                            let mut filters = filters;
                                            move |_| {
                                                ctx_clone.set_status(&mut filters, Some(SubscriberStatus::Unsubscribed));
                                            }
                                        },
                                        "Unsubscribed"
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
