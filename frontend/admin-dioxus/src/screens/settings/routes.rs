use dioxus::prelude::*;
use oxstore::{ListQuery, ListStore, Order};
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::components::table::list_empty_state::ListEmptyState;
use crate::components::table::list_toolbar::ListToolbarProps;
use crate::components::table::skeleton_table_rows::{
    SkeletonCellConfig, SkeletonTableRows, UICellType,
};
use crate::containers::page_header::PageHeaderProps;
use crate::hooks::{use_list_screen_with_handlers, ListScreenConfig};
use crate::utils::dates::format_short_date_dt;
use ruxlog_shared::store::{
    use_admin_routes, AdminRoutesListQuery, BlockRoutePayload, RouteStatus, UpdateRoutePayload,
    UpdateSyncIntervalPayload,
};

#[component]
pub fn RoutesSettingsScreen() -> Element {
    let routes_state = use_admin_routes();
    let filters = use_signal(|| AdminRoutesListQuery::new());
    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "route_pattern".to_string(),
            default_sort_order: Order::Asc,
        }),
        filters,
    );

    let mut pattern = use_signal(|| "".to_string());
    let mut reason = use_signal(|| "".to_string());
    let mut interval_input = use_signal(|| "".to_string());

    use_effect({
        let list_state = list_state;
        move || {
            let query = filters();
            let _tick = list_state.reload_tick();
            let routes_state = routes_state;
            spawn(async move {
                routes_state.fetch_list_with_query(query).await;
                routes_state.fetch_sync_interval().await;
            });
        }
    });

    let block_route = move |event: FormEvent| {
        event.prevent_default();
        let pattern_value = pattern().trim().to_string();
        if pattern_value.is_empty() {
            return;
        }

        let payload = BlockRoutePayload {
            pattern: pattern_value.clone(),
            reason: {
                let r = reason().trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            },
        };
        let routes_state = routes_state;
        let mut pattern = pattern;
        let mut reason = reason;
        spawn(async move {
            routes_state.block(payload).await;
            pattern.set(String::new());
            reason.set(String::new());
        });
    };

    let routes_frame = (routes_state.list)();
    let interval_frame = (routes_state.sync_interval)();
    let list_loading = routes_frame.is_loading();
    let interval_loading = interval_frame.is_loading();

    let (rows, current_page) = if let Some(paginated) = &routes_frame.data {
        (paginated.data.clone(), paginated.page)
    } else {
        (Vec::<RouteStatus>::new(), 1)
    };

    if interval_input().is_empty() {
        if let Some(interval_data) = &interval_frame.data {
            interval_input.set(interval_data.interval_secs.to_string());
        }
    }

    let filters_snapshot = filters.read().clone();
    let mut filtered_rows: Vec<RouteStatus> = rows
        .into_iter()
        .filter(|route| {
            let mut keep = true;
            if let Some(search) = filters_snapshot.search.as_ref() {
                let needle = search.to_lowercase();
                keep &= route.route_pattern.to_lowercase().contains(&needle)
                    || route
                        .reason
                        .as_ref()
                        .map(|reason| reason.to_lowercase().contains(&needle))
                        .unwrap_or(false);
            }
            if let Some(blocked) = filters_snapshot.is_blocked {
                keep &= route.is_blocked == blocked;
            }
            keep
        })
        .collect();

    let sort_field = list_state.sort_field();
    let sort_order = list_state.sort_order();

    filtered_rows.sort_by(|a, b| match sort_field.as_str() {
        "created_at" => a.created_at.cmp(&b.created_at),
        "updated_at" => a.updated_at.cmp(&b.updated_at),
        "is_blocked" => a.is_blocked.cmp(&b.is_blocked),
        _ => a.route_pattern.cmp(&b.route_pattern),
    });
    if matches!(sort_order, Order::Desc) {
        filtered_rows.reverse();
    }

    let headers = vec![
        HeaderColumn::new(
            "Pattern",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("route_pattern"),
        ),
        HeaderColumn::new(
            "Status",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("is_blocked"),
        ),
        HeaderColumn::new(
            "Reason",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm",
            None,
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
        HeaderColumn::new("", false, "w-32 py-2 px-3 text-right", None),
    ];

    let handle_status_select = {
        let mut filters = filters;
        move |value: String| {
            let mut query = filters.peek().clone();
            query.set_page(1);
            query.is_blocked = match value.to_lowercase().as_str() {
                "blocked" => Some(true),
                "allowed" => Some(false),
                _ => None,
            };
            filters.set(query);
        }
    };

    let sync_now = move |_| {
        let routes_state = routes_state;
        spawn(async move {
            routes_state.sync().await;
        });
    };

    let update_interval = move |_| {
        let value = interval_input().trim().to_string();
        if value.is_empty() {
            return;
        }
        if let Ok(interval_secs) = value.parse::<u64>() {
            let routes_state = routes_state;
            let payload = UpdateSyncIntervalPayload { interval_secs };
            spawn(async move {
                routes_state.update_sync_interval(payload).await;
                routes_state.fetch_sync_interval().await;
            });
        }
    };

    let pause_interval = move |_| {
        let routes_state = routes_state;
        spawn(async move {
            routes_state.pause_sync_interval().await;
        });
    };

    let resume_interval = move |_| {
        let routes_state = routes_state;
        spawn(async move {
            routes_state.resume_sync_interval().await;
        });
    };

    let restart_interval = move |_| {
        let routes_state = routes_state;
        spawn(async move {
            routes_state.restart_sync_interval().await;
        });
    };

    let status_label = match filters_snapshot.is_blocked {
        Some(true) => "Blocked".to_string(),
        Some(false) => "Allowed".to_string(),
        None => "All".to_string(),
    };

    let toolbar = ListToolbarProps {
        search_value: list_state.search_input(),
        search_placeholder: "Search routes by pattern or reason".to_string(),
        disabled: list_loading,
        on_search_input: handlers.handle_search.clone(),
        status_selected: status_label,
        on_status_select: EventHandler::new(handle_status_select),
        status_options: Some(vec![
            "All".to_string(),
            "Blocked".to_string(),
            "Allowed".to_string(),
        ]),
    };

    let paused = interval_frame
        .data
        .as_ref()
        .map(|d| d.paused)
        .unwrap_or(false);
    let interval_label = interval_frame
        .data
        .as_ref()
        .map(|d| format!("{} seconds", d.interval_secs))
        .unwrap_or_else(|| "Loading...".to_string());

    let below_toolbar = rsx! {
        div { class: "grid gap-4 md:grid-cols-2",
            form { class: "bg-card border border-border rounded-lg p-4 space-y-3", onsubmit: block_route,
                div { class: "space-y-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Pattern" }
                    SimpleInput {
                        placeholder: Some("/admin/*".to_string()),
                        value: pattern(),
                        oninput: move |value| pattern.set(value),
                        class: Some("text-sm font-mono".to_string()),
                    }
                }
                div { class: "space-y-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Reason (optional)" }
                    SimpleInput {
                        placeholder: Some("Maintenance window".to_string()),
                        value: reason(),
                        oninput: move |value| reason.set(value),
                        class: Some("text-sm".to_string()),
                    }
                }
                Button { r#type: "submit", class: "w-full md:w-auto", "Block route" }
            }

            div { class: "bg-card border border-border rounded-lg p-4 space-y-4",
                div { class: "flex items-center justify-between",
                    h3 { class: "text-sm font-semibold", "Sync interval" }
                    Button {
                        variant: ButtonVariant::Outline,
                        class: "h-8 text-xs",
                        onclick: sync_now,
                        "Sync now"
                    }
                }
                div { class: "grid gap-2",
                    label { class: "text-xs font-medium text-muted-foreground", "Interval (seconds)" }
                    SimpleInput {
                        value: interval_input(),
                        oninput: move |value| interval_input.set(value),
                        class: Some("text-sm".to_string()),
                    }
                }
                p { class: "text-xs text-muted-foreground", "{interval_label}" }
                div { class: "flex flex-wrap gap-2",
                    Button {
                        variant: ButtonVariant::Secondary,
                        disabled: interval_loading,
                        onclick: update_interval,
                        "Update"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: interval_loading || paused,
                        onclick: pause_interval,
                        "Pause"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: interval_loading || !paused,
                        onclick: resume_interval,
                        "Resume"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: interval_loading,
                        onclick: restart_interval,
                        "Restart"
                    }
                }
            }
        }
    };

    let has_rows = !filtered_rows.is_empty();
    let mut pattern_for_empty = pattern;

    rsx! {
        DataTableScreen::<RouteStatus> {
            frame: routes_frame,
            header: Some(PageHeaderProps {
                title: "Route Blocker".to_string(),
                description: "Capture and manage dynamic route policies.".to_string(),
                actions: Some(rsx!{
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| handlers.handle_retry.call(()),
                        "Refresh"
                    }
                }),
                class: None,
                embedded: false,
            }),
            headers: Some(headers),
            current_sort_field: Some(list_state.sort_field()),
            on_sort: Some(handlers.handle_sort.clone()),
            error_title: Some("Failed to load routes".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(toolbar),
            below_toolbar: Some(below_toolbar),
            on_prev: move |_| { handlers.handle_prev.call(current_page); },
            on_next: move |_| { handlers.handle_next.call(current_page); },
            show_pagination: false,
            if filtered_rows.is_empty() {
                if list_loading && !has_rows {
                    SkeletonTableRows {
                        row_count: 5,
                        cells: vec![
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Badge, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Action, "py-2 px-3"),
                        ],
                    }
                } else {
                    tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                        td { colspan: "6", class: "py-12 px-4 text-center",
                            ListEmptyState {
                                title: "No routes recorded".to_string(),
                                description: "Routes are tracked automatically as they are accessed. You can also seed them manually.".to_string(),
                                clear_label: "Reload".to_string(),
                                create_label: "Block route".to_string(),
                                on_clear: move |_| { handlers.handle_retry.call(()); },
                                on_create: move |_| {
                                    if pattern_for_empty().is_empty() {
                                        pattern_for_empty.set("/admin/*".to_string());
                                    }
                                },
                            }
                        }
                    }
                }
            } else {
                {filtered_rows.into_iter().map(|route| {
                    rsx!{
                        RouteRow { route }
                    }
                })}
            }
        }
    }
}

#[component]
fn RouteRow(route: RouteStatus) -> Element {
    let routes_state = use_admin_routes();
    let status = if route.is_blocked {
        "Blocked"
    } else {
        "Allowed"
    };
    let status_variant = if route.is_blocked {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Secondary
    };
    let reason = route.reason.clone().unwrap_or_else(|| "-".to_string());

    let toggle_block = {
        let pattern = route.route_pattern.clone();
        let reason = route.reason.clone();
        let routes_state = routes_state;
        move |_| {
            let payload = UpdateRoutePayload {
                is_blocked: !route.is_blocked,
                reason: reason.clone(),
            };
            let routes_state = routes_state;
            let pattern = pattern.clone();
            spawn(async move {
                routes_state.update(pattern, payload).await;
            });
        }
    };

    let delete_route = {
        let pattern = route.route_pattern.clone();
        let routes_state = routes_state;
        move |_| {
            let pattern = pattern.clone();
            spawn(async move {
                routes_state.remove(pattern).await;
            });
        }
    };

    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "py-2 px-3 font-mono text-xs md:text-sm", "{route.route_pattern}" }
            td { class: "py-2 px-3",
                Badge { variant: status_variant, class: "text-[10px] uppercase tracking-wide", "{status}" }
            }
            td { class: "py-2 px-3 text-xs text-muted-foreground break-words", "{reason}" }
            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap", "{format_short_date_dt(&route.created_at)}" }
            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap", "{format_short_date_dt(&route.updated_at)}" }
            td { class: "py-2 px-3 text-right space-x-2",
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-8 px-2 text-xs",
                    onclick: toggle_block,
                    if route.is_blocked { "Unblock" } else { "Block" }
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    class: "h-8 px-2 text-xs",
                    onclick: delete_route,
                    "Delete"
                }
            }
        }
    }
}
