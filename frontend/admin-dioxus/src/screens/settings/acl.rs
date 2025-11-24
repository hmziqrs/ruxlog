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
use ruxlog_shared::store::{use_acl, AclCreatePayload, AclListQuery, AppConstant};

#[component]
pub fn AclSettingsScreen() -> Element {
    let acl_state = use_acl();
    let filters = use_signal(|| AclListQuery::new());
    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "key".to_string(),
            default_sort_order: Order::Asc,
        }),
        filters,
    );

    let mut key = use_signal(|| "".to_string());
    let mut value = use_signal(|| "".to_string());
    let mut value_type = use_signal(|| "".to_string());
    let mut description = use_signal(|| "".to_string());
    let mut is_sensitive = use_signal(|| false);

    use_effect({
        let list_state = list_state;
        move || {
            let query = filters();
            let _tick = list_state.reload_tick();
            let acl_state = acl_state;
            spawn(async move {
                acl_state.fetch_list_with_query(query).await;
            });
        }
    });

    let create_constant = move |event: FormEvent| {
        event.prevent_default();
        let k = key().trim().to_string();
        if k.is_empty() {
            return;
        }
        let payload = AclCreatePayload {
            key: k.clone(),
            value: value().trim().to_string(),
            value_type: if value_type().trim().is_empty() {
                None
            } else {
                Some(value_type().trim().to_string())
            },
            description: if description().trim().is_empty() {
                None
            } else {
                Some(description().trim().to_string())
            },
            is_sensitive: Some(is_sensitive()),
        };
        let acl_state = acl_state;
        let mut key = key;
        let mut value = value;
        let mut value_type = value_type;
        let mut description = description;
        let mut is_sensitive = is_sensitive;
        spawn(async move {
            acl_state.create(payload).await;
            key.set(String::new());
            value.set(String::new());
            value_type.set(String::new());
            description.set(String::new());
            is_sensitive.set(false);
        });
    };

    let list_frame = (acl_state.list)();
    let list_loading = list_frame.is_loading();

    let (rows, current_page, _total, _per_page) = if let Some(p) = &list_frame.data {
        (p.data.clone(), p.page, p.total, p.per_page)
    } else {
        (Vec::<AppConstant>::new(), 1, 0, 20)
    };

    let filters_snapshot = filters.read().clone();
    let mut filtered_rows: Vec<AppConstant> = rows
        .into_iter()
        .filter(|item| {
            let mut keep = true;
            if let Some(search) = filters_snapshot.search.as_ref() {
                let needle = search.to_lowercase();
                keep &= item.key.to_lowercase().contains(&needle)
                    || item
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&needle))
                        .unwrap_or(false);
            }
            if let Some(flag) = filters_snapshot.is_sensitive {
                keep &= item.is_sensitive == flag;
            }
            keep
        })
        .collect();

    let sort_field = list_state.sort_field();
    let sort_order = list_state.sort_order();

    filtered_rows.sort_by(|a, b| match sort_field.as_str() {
        "key" => a.key.cmp(&b.key),
        "value_type" => a.value_type.cmp(&b.value_type),
        "is_sensitive" => a.is_sensitive.cmp(&b.is_sensitive),
        "updated_at" => a.updated_at.cmp(&b.updated_at),
        "created_at" => a.created_at.cmp(&b.created_at),
        _ => a.key.cmp(&b.key),
    });
    if matches!(sort_order, Order::Desc) {
        filtered_rows.reverse();
    }

    let headers = vec![
        HeaderColumn::new(
            "Key",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("key"),
        ),
        HeaderColumn::new(
            "Value",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Type",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("value_type"),
        ),
        HeaderColumn::new(
            "Sensitive",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("is_sensitive"),
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
            let mut q = filters.peek().clone();
            q.set_page(1);
            q.is_sensitive = match value.to_lowercase().as_str() {
                "sensitive" => Some(true),
                "non-sensitive" => Some(false),
                _ => None,
            };
            filters.set(q);
        }
    };

    let toolbar = ListToolbarProps {
        search_value: list_state.search_input(),
        search_placeholder: "Search constants by key or description".to_string(),
        disabled: list_loading,
        on_search_input: handlers.handle_search.clone(),
        status_selected: match filters_snapshot.is_sensitive {
            Some(true) => "Sensitive".to_string(),
            Some(false) => "Non-sensitive".to_string(),
            None => "All".to_string(),
        },
        on_status_select: EventHandler::new(handle_status_select),
        status_options: Some(vec![
            "All".to_string(),
            "Sensitive".to_string(),
            "Non-sensitive".to_string(),
        ]),
    };

    let below_toolbar = rsx! {
        form { class: "bg-card border border-border rounded-lg p-4 space-y-3", onsubmit: create_constant,
            div { class: "grid gap-3 md:grid-cols-2",
                div { class: "space-y-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Key" }
                    SimpleInput {
                        placeholder: Some("S3_BUCKET".to_string()),
                        value: key(),
                        oninput: move |val| key.set(val),
                        class: Some("text-sm font-mono".to_string()),
                    }
                }
                div { class: "space-y-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Type (optional)" }
                    SimpleInput {
                        placeholder: Some("string | bool | int | json".to_string()),
                        value: value_type(),
                        oninput: move |val| value_type.set(val),
                        class: Some("text-sm".to_string()),
                    }
                }
            }
            div { class: "grid gap-3 md:grid-cols-2",
                div { class: "space-y-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Value" }
                    SimpleInput {
                        placeholder: Some("value".to_string()),
                        value: value(),
                        oninput: move |val| value.set(val),
                        class: Some("text-sm".to_string()),
                    }
                }
                div { class: "space-y-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Description (optional)" }
                    SimpleInput {
                        placeholder: Some("Describe this constant".to_string()),
                        value: description(),
                        oninput: move |val| description.set(val),
                        class: Some("text-sm".to_string()),
                    }
                }
            }
            div { class: "flex items-center gap-2",
                input {
                    r#type: "checkbox",
                    checked: is_sensitive(),
                    onchange: move |ev| {
                        is_sensitive.set(ev.value().parse::<bool>().unwrap_or(false));
                    }
                }
                span { class: "text-xs text-muted-foreground", "Mark as sensitive (value will be masked in UI)" }
            }
            Button { r#type: "submit", class: "w-full md:w-auto", "Create/Update constant" }
        }
    };

    let has_rows = !filtered_rows.is_empty();

    rsx! {
        DataTableScreen::<AppConstant> {
            frame: list_frame,
            header: Some(PageHeaderProps {
                title: "Application Constants".to_string(),
                description: "Manage ACL constants with DB + Redis cache.".to_string(),
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
            error_title: Some("Failed to load constants".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(toolbar),
            below_toolbar: Some(below_toolbar),
            on_prev: move |_| { handlers.handle_prev.call(current_page); },
            on_next: move |_| { handlers.handle_next.call(current_page); },
            show_pagination: true,
            if filtered_rows.is_empty() {
                if list_loading && !has_rows {
                    SkeletonTableRows {
                        row_count: 5,
                        cells: vec![
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Badge, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(UICellType::Action, "py-2 px-3"),
                        ],
                    }
                } else {
                    tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                        td { colspan: "6", class: "py-12 px-4 text-center",
                            ListEmptyState {
                                title: "No constants found".to_string(),
                                description: "Create a constant to manage it here.".to_string(),
                                clear_label: "Clear search".to_string(),
                                create_label: "Create constant".to_string(),
                                on_clear: move |_| { handlers.handle_clear.call(()); },
                                on_create: move |_| { key.set("NEW_KEY".to_string()); },
                            }
                        }
                    }
                }
            } else {
                {filtered_rows.into_iter().map(|constant| {
                    rsx!{
                        AclRow { constant }
                    }
                })}
            }
        }
    }
}

#[component]
fn AclRow(constant: AppConstant) -> Element {
    let acl_state = use_acl();
    let value_display = if constant.is_sensitive {
        "********".to_string()
    } else {
        constant.value.clone()
    };
    let type_display = constant
        .value_type
        .clone()
        .unwrap_or_else(|| "string".to_string());
    let badge_variant = if constant.is_sensitive {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Secondary
    };

    let delete_constant = {
        let key = constant.key.clone();
        let acl_state = acl_state;
        move |_| {
            let key = key.clone();
            spawn(async move {
                acl_state.remove(key).await;
            });
        }
    };

    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "py-2 px-3 font-mono text-xs md:text-sm break-all", "{constant.key}" }
            td { class: "py-2 px-3 text-xs md:text-sm break-all", "{value_display}" }
            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap", "{type_display}" }
            td { class: "py-2 px-3",
                Badge { variant: badge_variant, class: "text-[10px] uppercase tracking-wide", if constant.is_sensitive { "Sensitive" } else { "Public" } }
            }
            td { class: "py-2 px-3 text-xs text-muted-foreground whitespace-nowrap", "{format_short_date_dt(&constant.updated_at)}" }
            td { class: "py-2 px-3 text-right space-x-2",
                Button {
                    variant: ButtonVariant::Destructive,
                    class: "h-8 px-2 text-xs",
                    onclick: delete_constant,
                    "Delete"
                }
            }
        }
    }
}
