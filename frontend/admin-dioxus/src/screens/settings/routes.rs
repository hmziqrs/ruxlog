use dioxus::prelude::*;
use ruxlog_shared::store::{use_admin_routes, BlockRoutePayload, RouteStatus, UpdateRoutePayload};

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::containers::page_header::PageHeaderProps;
use oxstore::{PaginatedList, StateFrame};
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::{Button, ButtonVariant};

#[component]
pub fn RoutesSettingsScreen() -> Element {
    let routes_state = use_admin_routes();
    let mut pattern = use_signal(|| "".to_string());
    let mut reason = use_signal(|| "".to_string());

    use_effect(move || {
        spawn(async move {
            routes_state.list().await;
        });
    });

    let block_route = move |event: FormEvent| {
        event.prevent_default();
        let payload = BlockRoutePayload {
            pattern: pattern(),
            reason: if reason().is_empty() {
                None
            } else {
                Some(reason())
            },
        };
        let routes_state = routes_state;
        spawn(async move {
            routes_state.block(payload).await;
        });
    };

    let refresh = move |_| {
        let routes_state = routes_state;
        spawn(async move {
            routes_state.list().await;
        });
    };

    let rows: Vec<RouteStatus> = routes_state.list.read().data.clone().unwrap_or_default();
    let routes_frame = to_paginated_frame((routes_state.list)());

    // Define header columns
    let headers = vec![
        HeaderColumn::new(
            "Pattern",
            false,
            "p-3 text-left font-medium text-xs md:text-sm",
            None,
        ),
        HeaderColumn::new(
            "Status",
            false,
            "p-3 text-left font-medium text-xs md:text-sm",
            None,
        ),
        HeaderColumn::new(
            "Reason",
            false,
            "p-3 text-left font-medium text-xs md:text-sm",
            None,
        ),
        HeaderColumn::new(
            "Actions",
            false,
            "p-3 text-left font-medium text-xs md:text-sm",
            None,
        ),
    ];

    rsx! {
        DataTableScreen::<RouteStatus> {
            frame: routes_frame,
            header: Some(PageHeaderProps {
                title: "Route Settings".to_string(),
                description: "Block or unblock admin routes".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: refresh,
                        "Refresh"
                    }
                }),
                class: None,
                embedded: false,
            }),
            headers: Some(headers),
            current_sort_field: None,
            on_sort: None,
            show_pagination: false,
            on_prev: move |_| {},
            on_next: move |_| {},
            below_toolbar: Some(rsx! {
                form { class: "grid gap-3 md:grid-cols-3 items-end border border-border rounded-md p-3", onsubmit: block_route,
                    div { class: "space-y-1",
                        label { class: "text-sm font-medium", "Pattern" }
                        SimpleInput {
                            placeholder: Some("/admin/*".to_string()),
                            value: pattern(),
                            oninput: move |value| pattern.set(value),
                            class: Some("text-sm".to_string()),
                        }
                    }
                    div { class: "space-y-1",
                        label { class: "text-sm font-medium", "Reason" }
                        SimpleInput {
                            placeholder: Some("Maintenance".to_string()),
                            value: reason(),
                            oninput: move |value| reason.set(value),
                            class: Some("text-sm".to_string()),
                        }
                    }
                    Button {
                        r#type: "submit",
                        "Block route"
                    }
                }
            }),
            if rows.is_empty() {
                tr {
                    td { class: "p-4 text-center text-muted-foreground", colspan: "4",
                        "No routes configured."
                    }
                }
            } else {
                {rows.iter().cloned().map(|route| {
                    rsx! {
                        RouteRow { key: "{route.pattern}", route }
                    }
                })}
            }
        }
    }
}

#[component]
fn RouteRow(route: RouteStatus) -> Element {
    let routes_state = use_admin_routes();
    let status_label = if route.is_blocked {
        "Blocked"
    } else {
        "Allowed"
    };
    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "p-3 font-mono text-xs", "{route.pattern}" }
            td { class: "p-3", "{status_label}" }
            td { class: "p-3", "{route.reason.clone().unwrap_or_else(|| \"-\".to_string())}" }
            td { class: "p-3 space-x-2",
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-8 px-2 text-xs",
                    onclick: {
                        let routes_state = routes_state;
                        let pattern = route.pattern.clone();
                        let reason = route.reason.clone();
                        let is_blocked = route.is_blocked;
                        move |_| {
                            let payload = UpdateRoutePayload {
                                is_blocked: Some(!is_blocked),
                                reason: reason.clone(),
                            };
                            let pattern = pattern.clone();
                            spawn(async move { routes_state.update(pattern, payload).await; });
                        }
                    },
                    if route.is_blocked { "Unblock" } else { "Block" }
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    class: "h-8 px-2 text-xs",
                    onclick: {
                        let routes_state = routes_state;
                        let pattern = route.pattern.clone();
                        move |_| {
                            let pattern = pattern.clone();
                            spawn(async move { routes_state.remove(pattern).await; });
                        }
                    },
                    "Delete"
                }
            }
        }
    }
}

fn to_paginated_frame<T: Clone>(frame: StateFrame<Vec<T>>) -> StateFrame<PaginatedList<T>> {
    let data_vec = frame.data;
    let count = data_vec.as_ref().map(|d| d.len() as u64).unwrap_or(0);
    let data = data_vec.map(|items| PaginatedList {
        data: items,
        total: count,
        page: 1,
        per_page: count.max(1),
    });

    StateFrame {
        status: frame.status,
        data,
        meta: frame.meta,
        error: frame.error,
    }
}
