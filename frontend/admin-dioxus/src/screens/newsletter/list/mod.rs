mod components;
mod context;
mod utils;

use components::TableView;
use context::NewsletterListContext;

use dioxus::prelude::*;

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::components::table::list_toolbar::ListToolbarProps;
use crate::containers::page_header::PageHeaderProps;
use crate::hooks::{use_list_screen_with_handlers, ListScreenConfig};
use oxstore::{ListQuery, ListStore, Order};
use oxui::shadcn::button::Button;
use ruxlog_shared::store::{use_newsletter, NewsletterSubscriber, SubscriberListQuery};

#[component]
pub fn NewsletterSubscribersScreen() -> Element {
    let newsletter_state = use_newsletter();

    let filters = use_signal(|| SubscriberListQuery::new());

    let ctx = NewsletterListContext::new();
    use_context_provider(|| ctx.clone());

    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "created_at".to_string(),
            default_sort_order: Order::Desc,
        }),
        filters,
    );

    use_effect({
        let list_state = list_state;
        let mut selected_ids = ctx.selected_ids;
        move || {
            let q = filters();
            let _tick = list_state.reload_tick();
            let newsletter_state = newsletter_state;
            selected_ids.set(Vec::new());
            spawn(async move {
                newsletter_state.fetch_list_with_query(q).await;
            });
        }
    });

    let list = newsletter_state.subscribers.read();
    let list_loading = list.is_loading();

    let (subscribers, current_page) = if let Some(p) = &list.data {
        (p.data.clone(), p.page)
    } else {
        (Vec::new(), 1)
    };

    let has_data = !subscribers.is_empty();

    let active_filter_count = ctx.active_filter_count(&filters);

    let below_toolbar_content = rsx! {
        div { class: "flex flex-col gap-3",
            div { class: "flex items-center gap-2",
                components::FilterPopover { active_filter_count, filters }
            }
            if active_filter_count > 0 {
                components::ActiveFilters { active_filter_count, filters }
            }
        }
    };

    let bulk_actions = if !ctx.selected_ids.read().is_empty() {
        Some(rsx! {
            components::BulkActionsBar {}
        })
    } else {
        None
    };

    let headers = vec![
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
        HeaderColumn::new(
            "ID",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("id"),
        ),
        HeaderColumn::new(
            "Email",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("email"),
        ),
        HeaderColumn::new(
            "Status",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("status"),
        ),
        HeaderColumn::new(
            "Created",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("created_at"),
        ),
        HeaderColumn::new(
            "Updated",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("updated_at"),
        ),
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
    ];

    rsx! {
        DataTableScreen::<NewsletterSubscriber> {
            frame: (newsletter_state.subscribers)(),
            headers: Some(headers),
            current_sort_field: Some(list_state.sort_field()),
            on_sort: Some(handlers.handle_sort.clone()),
            header: Some(PageHeaderProps {
                title: "Newsletter Subscribers".to_string(),
                description: "Manage newsletter subscriptions and send campaigns.".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: move |_| {
                            // TODO: Navigate to send newsletter screen
                        },
                        "Send Newsletter"
                    }
                }),
                class: None,
                embedded: false,
            }),
            error_title: Some("Failed to load subscribers".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(ListToolbarProps {
                search_value: list_state.search_input(),
                search_placeholder: "Search subscribers by email".to_string(),
                disabled: list_loading,
                on_search_input: handlers.handle_search.clone(),
                status_selected: match filters.read().status {
                    Some(ruxlog_shared::store::SubscriberStatus::Confirmed) => "Confirmed".to_string(),
                    Some(ruxlog_shared::store::SubscriberStatus::Pending) => "Pending".to_string(),
                    Some(ruxlog_shared::store::SubscriberStatus::Unsubscribed) => "Unsubscribed".to_string(),
                    None => "All Status".to_string(),
                },
                on_status_select: EventHandler::new({
                    let mut filters = filters;
                    move |value: String| {
                        let mut q = filters.peek().clone();
                        q.set_page(1);
                        q.status = match value.as_str() {
                            "Confirmed" => Some(ruxlog_shared::store::SubscriberStatus::Confirmed),
                            "Pending" => Some(ruxlog_shared::store::SubscriberStatus::Pending),
                            "Unsubscribed" => Some(ruxlog_shared::store::SubscriberStatus::Unsubscribed),
                            _ => None,
                        };
                        filters.set(q);
                    }
                }),
                status_options: None,
            }),
            below_toolbar: Some(rsx!{
                div { class: "space-y-3",
                    {below_toolbar_content}
                    {bulk_actions}
                }
            }),
            on_prev: move |_| { handlers.handle_prev.call(current_page); },
            on_next: move |_| { handlers.handle_next.call(current_page); },

            TableView {
                subscribers: subscribers.clone(),
                list_loading,
                has_data,
                on_clear: handlers.handle_clear
            }
        }
    }
}
