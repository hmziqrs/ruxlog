use dioxus::prelude::*;
use ruxlog_shared::store::{
    use_admin_routes, BlockRoutePayload, RouteStatus, UpdateRoutePayload,
};

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
            reason: if reason().is_empty() { None } else { Some(reason()) },
        };
        let routes_state = routes_state;
        spawn(async move {
            routes_state.block(payload).await;
        });
    };

    let rows: Vec<RouteStatus> = routes_state
        .list
        .read()
        .data
        .clone()
        .unwrap_or_default();

    rsx! {
        div { class: "p-6 space-y-5",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-2xl font-semibold", "Route Settings" }
                    p { class: "text-sm text-muted-foreground", "Block or unblock admin routes." }
                }
                button {
                    class: "inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-accent",
                    onclick: move |_| {
                        let routes_state = routes_state;
                        spawn(async move { routes_state.list().await; });
                    },
                    "Refresh"
                }
            }

            form { class: "grid gap-3 md:grid-cols-3 items-end border border-border rounded-md p-3", onsubmit: block_route,
                div { class: "space-y-1",
                    label { class: "text-sm font-medium", "Pattern" }
                    input {
                        class: "w-full rounded-md border border-border px-3 py-2 text-sm",
                        placeholder: "/admin/*",
                        value: "{pattern}",
                        oninput: move |e| pattern.set(e.value()),
                        required: true
                    }
                }
                div { class: "space-y-1",
                    label { class: "text-sm font-medium", "Reason" }
                    input {
                        class: "w-full rounded-md border border-border px-3 py-2 text-sm",
                        placeholder: "Maintenance",
                        value: "{reason}",
                        oninput: move |e| reason.set(e.value()),
                    }
                }
                button {
                    class: "inline-flex items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:opacity-90",
                    r#type: "submit",
                    "Block route"
                }
            }

            div { class: "overflow-auto bg-transparent border border-border rounded-lg",
                table { class: "w-full text-sm",
                    thead { class: "bg-transparent",
                        tr {
                            th { class: "p-3 text-left", "Pattern" }
                            th { class: "p-3 text-left", "Status" }
                            th { class: "p-3 text-left", "Reason" }
                            th { class: "p-3 text-left", "Actions" }
                        }
                    }
                    tbody {
                        if rows.is_empty() {
                            tr {
                                td { class: "p-4 text-center text-muted-foreground", colspan: "4",
                                    "No routes configured."
                                }
                            }
                        } else {
                            for route in rows {
                                RouteRow { key: "{route.pattern}", route }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RouteRow(route: RouteStatus) -> Element {
    let routes_state = use_admin_routes();
    let status_label = if route.is_blocked { "Blocked" } else { "Allowed" };
    rsx! {
        tr { class: "border-t border-border",
            td { class: "p-3 font-mono text-xs", "{route.pattern}" }
            td { class: "p-3", "{status_label}" }
            td { class: "p-3", "{route.reason.clone().unwrap_or_else(|| \"-\".to_string())}" }
            td { class: "p-3 space-x-2",
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs hover:bg-accent",
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
                button {
                    class: "rounded-md border border-border px-2 py-1 text-xs text-red-600 hover:bg-red-50",
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
