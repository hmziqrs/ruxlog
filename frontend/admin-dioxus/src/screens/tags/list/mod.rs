use dioxus::prelude::*;

use crate::components::{
    DataTableScreen, HeaderColumn, ListEmptyState, ListToolbarProps, PageHeaderProps,
    SkeletonCellConfig, SkeletonTableRows,
};
use crate::hooks::{use_list_screen_with_handlers, ListScreenConfig};
use crate::router::Route;
use crate::store::{use_tag, ListQuery, ListStore, Tag, TagsListQuery};
use crate::types::Order;
use crate::ui::shadcn::{
    Badge, BadgeVariant, Button, ButtonVariant, Checkbox, DropdownMenu, DropdownMenuContent,
    DropdownMenuItem, DropdownMenuTrigger,
};
use crate::utils::dates::format_short_date_dt;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdEllipsis, Icon};

#[component]
pub fn TagsListScreen() -> Element {
    let nav = use_navigator();
    let tags_state = use_tag();

    let filters = use_signal(|| TagsListQuery::new());
    // Local selection state for the current page
    let selected_ids = use_signal(|| Vec::<i32>::new());

    // Use the enhanced hook that creates handlers for us
    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "name".to_string(),
            default_sort_order: Order::Asc,
        }),
        filters,
    );

    // Effect to load data when filters change - using the trait method
    use_effect({
        let list_state = list_state;
        let mut selected_ids = selected_ids;
        move || {
            let q = filters();
            let _tick = list_state.reload_tick();
            let tags_state = tags_state;
            // Clear any selection on query changes (page, search, filters, sorts)
            selected_ids.set(Vec::new());
            spawn(async move {
                tags_state.fetch_list_with_query(q).await;
            });
        }
    });

    let list = tags_state.list.read();
    let list_loading = list.is_loading();
    let _list_failed = list.is_failed();

    let (tags, current_page) = if let Some(p) = &list.data {
        (p.data.clone(), p.page)
    } else {
        (Vec::<Tag>::new(), 1)
    };

    let has_data = !tags.is_empty();

    // Define header columns (prepend a blank cell for the selection checkbox column)
    let headers = vec![
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
        HeaderColumn::new(
            "Name",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("name"),
        ),
        HeaderColumn::new(
            "Slug",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("slug"),
        ),
        HeaderColumn::new(
            "Description",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm",
            None,
        ),
        HeaderColumn::new(
            "Posts",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
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
        HeaderColumn::new(
            "Status",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
    ];

    // Custom status filter handler for tags - the rest is handled by the enhanced hook
    let handle_status_select = {
        let mut filters = filters;
        move |value: String| {
            let mut q = filters.peek().clone();
            q.set_page(1);
            q.is_active = match value.as_str() {
                "Active" | "active" => Some(true),
                "Inactive" | "inactive" => Some(false),
                _ => None,
            };
            filters.set(q);
        }
    };

    rsx! {
        DataTableScreen::<Tag> {
            frame: (tags_state.list)(),
            header: Some(PageHeaderProps {
                title: "Tags".to_string(),
                description: "Organize your content with tags".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: move |_| { nav.push(Route::TagsAddScreen {}); },
                        "New Tag"
                    }
                }),
                class: None,
                embedded: false,
            }),
            headers: Some(headers),
            current_sort_field: Some(list_state.sort_field()),
            on_sort: Some(handlers.handle_sort.clone()),
            error_title: Some("Failed to load tags".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(ListToolbarProps {
                search_value: list_state.search_input(),
                search_placeholder: "Search tags by name, description, or slug".to_string(),
                disabled: list_loading,
                on_search_input: handlers.handle_search.clone(),
                status_selected: match filters.read().is_active {
                    Some(true) => "Active".to_string(),
                    Some(false) => "Inactive".to_string(),
                    None => "All".to_string(),
                },
                on_status_select: EventHandler::new(handle_status_select),
            }),
            on_prev: move |_| { handlers.handle_prev.call(current_page); },
            on_next: move |_| { handlers.handle_next.call(current_page); },
            // Render selection actions between toolbar and table (below_toolbar slot)
            below_toolbar: if !selected_ids.read().is_empty() {
                Some(rsx! {
                    div { class: "w-full flex items-center justify-between bg-transparent border border-zinc-200 dark:border-zinc-800 rounded-md px-4 py-3 shadow-sm",
                        span { class: "text-sm text-muted-foreground", "{selected_ids.read().len()} selected" }
                        div { class: "flex items-center gap-2",
                            Button { variant: ButtonVariant::Outline, class: "h-8",
                                onclick: {
                                    let mut selected_ids = selected_ids;
                                    move |_| { selected_ids.set(Vec::new()); }
                                },
                                "Activate"
                            }
                            Button { variant: ButtonVariant::Outline, class: "h-8",
                                onclick: {
                                    let mut selected_ids = selected_ids;
                                    move |_| { selected_ids.set(Vec::new()); }
                                },
                                "Deactivate"
                            }
                            Button { variant: ButtonVariant::Outline, class: "h-8 text-red-600 border-red-200 dark:border-red-800 dark:hover:bg-red-950/20 hover:bg-red-50",
                                onclick: {
                                    let mut selected_ids = selected_ids;
                                    move |_| { selected_ids.set(Vec::new()); }
                                },
                                "Delete"
                            }
                        }
                    }
                })
            } else { None },
            // Table body content only - headers are now handled by DataTableScreen
            if tags.is_empty() {
                if list_loading && !has_data {
                    SkeletonTableRows {
                        row_count: 6,
                        cells: vec![
                            // Selection checkbox placeholder
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "w-12 py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Badge, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Action, "py-2 px-3"),
                        ],
                    }
                } else {
                    tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                        td { colspan: "9", class: "py-12 px-4 text-center",
                            ListEmptyState {
                                title: "No tags found".to_string(),
                                description: "Try adjusting your search or create a new tag to get started.".to_string(),
                                clear_label: "Clear search".to_string(),
                                create_label: "Create your first tag".to_string(),
                                on_clear: move |_| { handlers.handle_clear.call(()); },
                                on_create: move |_| { nav.push(Route::TagsAddScreen {}); },
                            }
                        }
                    }
                }
            } else {
                {tags.iter().cloned().map(|tag| {
                    let tag_id = tag.id;
                    rsx! {
                        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                            // Selection checkbox cell
                            td { class: "py-2 px-3 w-12 text-xs md:text-sm",
                                Checkbox {
                                    checked: selected_ids.read().contains(&tag_id),
                                    onchange: Some(EventHandler::new({
                                        let mut selected_ids = selected_ids;
                                        move |checked: bool| {
                                            let mut current = selected_ids.peek().clone();
                                            if checked {
                                                if !current.contains(&tag_id) {
                                                    current.push(tag_id);
                                                }
                                            } else {
                                                current.retain(|&id| id != tag_id);
                                            }
                                            selected_ids.set(current);
                                        }
                                    })),
                                }
                            }
                            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap",
                                span { class: "font-medium leading-none truncate", "{tag.name}" }
                            }
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                                span { class: "truncate font-mono", "{tag.slug}" }
                            }
                            td { class: "max-w-xs py-2 px-3 text-xs md:text-sm text-muted-foreground",
                                span { class: "truncate", {tag.description.clone().unwrap_or("—".to_string())} }
                            }
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap", "0" }
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap", "{format_short_date_dt(&tag.created_at)}" }
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap", "{format_short_date_dt(&tag.updated_at)}" }
                            td { class: "py-2 px-3 text-xs md:text-sm",
                                if tag.is_active {
                                    Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Active" }
                                } else {
                                    Badge { variant: BadgeVariant::Secondary, class: "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400", "Inactive" }
                                }
                            }
                            td { class: "py-2 px-3 text-xs md:text-sm",
                                DropdownMenu {
                                    DropdownMenuTrigger {
                                        Button { variant: ButtonVariant::Ghost, class: "h-8 w-8 p-0 bg-transparent hover:bg-muted/50", div { class: "w-4 h-4", Icon { icon: LdEllipsis {} } } }
                                    }
                                    DropdownMenuContent { class: "bg-background border-zinc-200 dark:border-zinc-800",
                                        DropdownMenuItem { onclick: move |_| { nav.push(Route::TagsEditScreen { id: tag_id }); }, "Edit" }
                                        DropdownMenuItem { onclick: move |_| { nav.push(Route::PostsListScreen {}); }, "View Posts" }
                                        DropdownMenuItem { class: "text-red-600", onclick: move |_| {
                                                let id = tag_id;
                                                spawn({  async move {
                                                    tags_state.remove(id).await;
                                                }});
                                            }, "Delete" }
                                    }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
