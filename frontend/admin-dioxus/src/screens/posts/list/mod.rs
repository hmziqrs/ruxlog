mod components;
mod context;
mod utils;

use components::{GridView, TableView};
use context::{PostListContext, ViewMode};

use dioxus::prelude::*;

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::components::table::list_toolbar::ListToolbarProps;
use crate::containers::page_header::PageHeaderProps;
use crate::hooks::{use_list_screen_with_handlers, ListScreenConfig};
use crate::router::Route;
use ruxlog_shared::store::{use_categories, use_post, use_tag, use_user, Post, PostListQuery};
use oxui::shadcn::button::{Button, ButtonSize, ButtonVariant};
use oxstore::{ListQuery, ListStore, Order};

use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdGrid3x3, LdLayoutList},
    Icon,
};

#[component]
pub fn PostsListScreen() -> Element {
    let nav = use_navigator();
    let posts_state = use_post();
    let categories_state = use_categories();
    let tags_state = use_tag();
    let users_state = use_user();

    // Use direct filter signal instead of context-based filters
    let filters = use_signal(|| PostListQuery::new());

    // Initialize context for UI-specific state (view mode, selections)
    let ctx = PostListContext::new();
    use_context_provider(|| ctx.clone());

    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "created_at".to_string(),
            default_sort_order: Order::Desc,
        }),
        filters,
    );

    // Load filter data on mount
    use_effect(move || {
        spawn(async move {
            categories_state.list().await;
            tags_state.list().await;
            users_state.list().await;
        });
    });

    // Effect to load posts when filters change - using the trait method
    use_effect({
        let list_state = list_state;
        let mut selected_ids = ctx.selected_ids;
        move || {
            let q = filters();
            let _tick = list_state.reload_tick();
            let posts_state = posts_state;
            // Clear any selection on query changes (page, search, filters, sorts)
            selected_ids.set(Vec::new());
            spawn(async move {
                posts_state.fetch_list_with_query(q).await;
            });
        }
    });

    let list = posts_state.list.read();
    let list_loading = list.is_loading();

    let (posts, current_page) = if let Some(p) = &list.data {
        (p.data.clone(), p.page)
    } else {
        (Vec::new(), 1)
    };

    let has_data = !posts.is_empty();

    // Calculate active filter count from the query filters
    let active_filter_count = ctx.active_filter_count(&filters);

    // Custom view mode switcher for the header actions
    let view_mode_switcher = {
        let current_mode = *ctx.view_mode.read();
        let mut ctx_clone1 = ctx.clone();
        let mut ctx_clone2 = ctx.clone();
        rsx! {
            div { class: "flex items-center gap-1 border border-zinc-200 dark:border-zinc-800 rounded-md p-1",
                Button {
                    variant: if current_mode == ViewMode::Grid {
                        ButtonVariant::Default
                    } else {
                        ButtonVariant::Ghost
                    },
                    size: ButtonSize::Sm,
                    class: "h-8 w-8 p-0",
                    onclick: move |_| { ctx_clone1.view_mode.set(ViewMode::Grid); },
                    Icon { icon: LdGrid3x3 {}, class: "w-4 h-4" }
                }
                Button {
                    variant: if current_mode == ViewMode::Table {
                        ButtonVariant::Default
                    } else {
                        ButtonVariant::Ghost
                    },
                    size: ButtonSize::Sm,
                    class: "h-8 w-8 p-0",
                    onclick: move |_| { ctx_clone2.view_mode.set(ViewMode::Table); },
                    Icon { icon: LdLayoutList {}, class: "w-4 h-4" }
                }
            }
        }
    };

    // Below toolbar content - filters and active filter badges
    let below_toolbar_content = rsx! {
        div { class: "flex flex-col gap-3",
            // Filter controls row
            div { class: "flex items-center gap-2",
                components::FilterPopover { active_filter_count, filters }
            }
            // Active filter badges
            if active_filter_count > 0 {
                components::ActiveFilters { active_filter_count, filters }
            }
        }
    };

    // Bulk actions if items selected
    let bulk_actions = if !ctx.selected_ids.read().is_empty() {
        Some(rsx! {
            components::BulkActionsBar {}
        })
    } else {
        None
    };

    // Define table headers
    let headers = vec![
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
        HeaderColumn::new(
            "Title",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("title"),
        ),
        HeaderColumn::new(
            "Author",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("author_id"),
        ),
        HeaderColumn::new(
            "Category",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("category_id"),
        ),
        HeaderColumn::new(
            "Status",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Stats",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Published",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("published_at"),
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
        DataTableScreen::<Post> {
            frame: (posts_state.list)(),
            headers: if *ctx.view_mode.read() == ViewMode::Table { Some(headers) } else { None },
            current_sort_field: if *ctx.view_mode.read() == ViewMode::Table { Some(list_state.sort_field()) } else { None },
            on_sort: if *ctx.view_mode.read() == ViewMode::Table { Some(handlers.handle_sort.clone()) } else { None },
            header: Some(PageHeaderProps {
                title: "Posts".to_string(),
                description: "Manage and view your blog posts. Create, edit, and organize content.".to_string(),
                actions: Some(rsx!{
                    div { class: "flex items-center gap-2",
                        {view_mode_switcher}
                        Button {
                            onclick: move |_| { nav.push(Route::PostsAddScreen {}); },
                            "Create Post"
                        }
                    }
                }),
                class: None,
                embedded: false,
            }),
            error_title: Some("Failed to load posts".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(ListToolbarProps {
                search_value: list_state.search_input(),
                search_placeholder: "Search posts by title, content, or author".to_string(),
                disabled: list_loading,
                on_search_input: handlers.handle_search.clone(),
                status_selected: match filters.read().status {
                    Some(ruxlog_shared::store::PostStatus::Published) => "Published".to_string(),
                    Some(ruxlog_shared::store::PostStatus::Draft) => "Draft".to_string(),
                    Some(ruxlog_shared::store::PostStatus::Archived) => "Archived".to_string(),
                    None => "All Status".to_string(),
                },
                on_status_select: EventHandler::new({
                    let mut filters = filters;
                    move |value: String| {
                        let mut q = filters.peek().clone();
                        q.set_page(1);
                        q.status = match value.as_str() {
                            "Published" => Some(ruxlog_shared::store::PostStatus::Published),
                            "Draft" => Some(ruxlog_shared::store::PostStatus::Draft),
                            "Archived" => Some(ruxlog_shared::store::PostStatus::Archived),
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

            // Render based on view mode
            if *ctx.view_mode.read() == ViewMode::Table {
                TableView {
                    posts: posts.clone(),
                    list_loading,
                    has_data,
                    on_clear: handlers.handle_clear
                }
            } else {
                GridView {
                    posts: posts.clone(),
                    list_loading,
                    has_data
                }
            }
        }
    }
}
