mod components;
mod context;

use components::{BulkActionsBar, TableView};
use context::CommentListContext;

use dioxus::prelude::*;

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::components::table::list_toolbar::ListToolbarProps;
use crate::containers::page_header::PageHeaderProps;
use crate::hooks::{use_list_screen_with_handlers, ListScreenConfig};
use ruxlog_shared::store::{
    use_comments, use_post, use_user, Comment, CommentListQuery, FlagFilter, HiddenFilter,
};
use oxui::shadcn::combobox::{Combobox, ComboboxItem};
use oxstore::{ListQuery, ListStore, Order};

#[component]
pub fn CommentsListScreen() -> Element {
    let comments_state = use_comments();
    let posts_state = use_post();
    let users_state = use_user();

    // Use direct filter signal
    let filters = use_signal(|| CommentListQuery::new());

    // Initialize context for UI-specific state (selections, filter state)
    let ctx = CommentListContext::new();
    use_context_provider(|| ctx.clone());

    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "created_at".to_string(),
            default_sort_order: Order::Desc,
        }),
        filters,
    );

    // Load filter data on mount (posts and users for combobox)
    use_effect(move || {
        spawn(async move {
            posts_state.list().await;
            users_state.list().await;
        });
    });

    // Effect to load comments when filters change
    use_effect({
        let list_state = list_state;
        let mut selected_ids = ctx.selected_ids;
        move || {
            let q = filters();
            let _tick = list_state.reload_tick();
            let comments_state = comments_state;
            // Clear any selection on query changes
            selected_ids.set(Vec::new());
            spawn(async move {
                comments_state.fetch_list_with_query(q).await;
            });
        }
    });

    let list = comments_state.list.read();
    let list_loading = list.is_loading();

    let (comments, current_page) = if let Some(p) = &list.data {
        (p.data.clone(), p.page)
    } else {
        (Vec::new(), 1)
    };

    let has_data = !comments.is_empty();

    // Bulk actions if items selected
    let bulk_actions = if !ctx.selected_ids.read().is_empty() {
        Some(rsx! {
            BulkActionsBar {}
        })
    } else {
        None
    };

    // Build user combobox items
    let users_list = users_state.list.read();
    let user_items: Vec<ComboboxItem> = users_list
        .data
        .clone()
        .map(|paginated| paginated.data)
        .unwrap_or_default()
        .into_iter()
        .map(|u| ComboboxItem {
            value: u.id.to_string(),
            label: u.name,
        })
        .collect();

    // Build post combobox items
    let posts_list = posts_state.list.read();
    let post_items: Vec<ComboboxItem> = posts_list
        .data
        .clone()
        .map(|paginated| paginated.data)
        .unwrap_or_default()
        .into_iter()
        .map(|p| ComboboxItem {
            value: p.id.to_string(),
            label: p.title,
        })
        .collect();

    // Filters section below toolbar
    let below_toolbar_content = rsx! {
        div { class: "space-y-3",
            // Filter row
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-3",
                div { class: "flex flex-col gap-1.5 md:max-w-xs",
                    label { class: "text-sm font-medium text-zinc-700 dark:text-zinc-300", "Filter by User" }
                    Combobox {
                        items: user_items,
                        placeholder: "Select user...".to_string(),
                        value: (*ctx.selected_user_id.read()).map(|id| id.to_string()),
                        width: "w-full md:w-64".to_string(),
                        onvaluechange: Some(EventHandler::new({
                            let mut ctx = ctx.clone();
                            let mut filters = filters;
                            move |val: Option<String>| {
                                let user_id = val.and_then(|v| v.parse::<i32>().ok());
                                ctx.set_user_filter(&mut filters, user_id);
                            }
                        })),
                    }
                }
                div { class: "flex flex-col gap-1.5 md:max-w-xs",
                    label { class: "text-sm font-medium text-zinc-700 dark:text-zinc-300", "Filter by Post" }
                    Combobox {
                        items: post_items,
                        placeholder: "Select post...".to_string(),
                        value: (*ctx.selected_post_id.read()).map(|id| id.to_string()),
                        width: "w-full md:w-64".to_string(),
                        onvaluechange: Some(EventHandler::new({
                            let mut ctx = ctx.clone();
                            let mut filters = filters;
                            move |val: Option<String>| {
                                let post_id = val.and_then(|v| v.parse::<i32>().ok());
                                ctx.set_post_filter(&mut filters, post_id);
                            }
                        })),
                    }
                }
                div { class: "flex flex-col gap-1.5 md:max-w-xs",
                    label { class: "text-sm font-medium text-zinc-700 dark:text-zinc-300", "Filter by Flags" }
                    Combobox {
                        items: vec![
                            ComboboxItem { value: "all".to_string(), label: "All".to_string() },
                            ComboboxItem { value: "flagged".to_string(), label: "Flagged".to_string() },
                            ComboboxItem { value: "not_flagged".to_string(), label: "Not flagged".to_string() },
                        ],
                        placeholder: "Select flag status...".to_string(),
                        value: Some(match *ctx.selected_flag_filter.read() {
                            FlagFilter::All => "all".to_string(),
                            FlagFilter::Flagged => "flagged".to_string(),
                            FlagFilter::NotFlagged => "not_flagged".to_string(),
                        }),
                        width: "w-full md:w-64".to_string(),
                        onvaluechange: Some(EventHandler::new({
                            let mut ctx = ctx.clone();
                            let mut filters = filters;
                            move |val: Option<String>| {
                                let filter = match val.as_deref() {
                                    Some("flagged") => FlagFilter::Flagged,
                                    Some("not_flagged") => FlagFilter::NotFlagged,
                                    _ => FlagFilter::All,
                                };
                                ctx.set_flag_filter(&mut filters, filter);
                            }
                        })),
                    }
                }
            }
            {bulk_actions}
        }
    };

    // Define table headers with sortable fields
    let headers = vec![
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
        HeaderColumn::new(
            "Comment ID",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Post",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("post_id"),
        ),
        HeaderColumn::new(
            "Author",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("user_id"),
        ),
        HeaderColumn::new(
            "Content",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Status",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("hidden"),
        ),
        HeaderColumn::new(
            "Created",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("created_at"),
        ),
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
    ];

    rsx! {
        DataTableScreen::<Comment> {
            frame: (comments_state.list)(),
            headers: Some(headers),
            current_sort_field: Some(list_state.sort_field()),
            on_sort: Some(handlers.handle_sort.clone()),
            header: Some(PageHeaderProps {
                title: "Comments".to_string(),
                description: "Moderate comments and flags".to_string(),
                actions: None,
                class: None,
                embedded: false,
            }),
            error_title: Some("Failed to load comments".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(ListToolbarProps {
                search_value: list_state.search_input(),
                search_placeholder: "Search comments...".to_string(),
                disabled: list_loading,
                on_search_input: handlers.handle_search.clone(),
                status_selected: match filters.read().hidden_filter {
                    Some(HiddenFilter::All) => "All".to_string(),
                    Some(HiddenFilter::Hidden) => "Hidden".to_string(),
                    Some(HiddenFilter::Visible) | None => "Visible".to_string(),
                },
                on_status_select: EventHandler::new({
                    let mut filters = filters;
                    move |value: String| {
                        let mut q = filters.peek().clone();
                        q.set_page(1);
                        let normalized = value.to_lowercase();
                        q.hidden_filter = match normalized.as_str() {
                            "all" => Some(HiddenFilter::All),
                            "hidden" | "hidden only" => Some(HiddenFilter::Hidden),
                            _ => Some(HiddenFilter::Visible),
                        };
                        filters.set(q);
                    }
                }),
                status_options: Some(vec![
                    "Visible".to_string(),
                    "Hidden".to_string(),
                    "All".to_string(),
                ]),
            }),
            below_toolbar: Some(rsx!{
                {below_toolbar_content}
            }),
            on_prev: move |_| { handlers.handle_prev.call(current_page); },
            on_next: move |_| { handlers.handle_next.call(current_page); },

            TableView {
                comments: comments.clone(),
                list_loading,
                has_data,
                on_clear: handlers.handle_clear,
                on_refresh: EventHandler::new({
                    let comments_state = comments_state;
                    let filters = filters;
                    move |_| {
                        let comments_state = comments_state;
                        let query = filters();
                        spawn(async move {
                            comments_state.fetch_list_with_query(query).await;
                        });
                    }
                })
            }
        }
    }
}
