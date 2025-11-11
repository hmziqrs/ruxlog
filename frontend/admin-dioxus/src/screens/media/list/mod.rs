use dioxus::prelude::*;

use crate::components::{
    DataTableScreen, HeaderColumn, ListEmptyState, ListToolbarProps, LoadingOverlay,
    MediaUsageDialog, PageHeaderProps, SkeletonCellConfig, SkeletonTableRows,
};
use crate::hooks::{
    use_list_screen_with_handlers, use_state_frame_map_toast, ListScreenConfig,
    StateFrameToastConfig,
};
use crate::router::Route;
use crate::store::{use_media, ListQuery, ListStore, Media, MediaListQuery, MediaReference};
use crate::types::Order;
use crate::ui::shadcn::{
    Badge, Button, ButtonVariant, Checkbox, DropdownMenu, DropdownMenuContent, DropdownMenuItem,
    DropdownMenuTrigger,
};
use crate::utils::dates::format_short_date_dt;
use crate::utils::file_helpers::{format_file_size, is_image};
use hmziq_dioxus_free_icons::{icons::ld_icons::LdEllipsis, Icon};

#[component]
pub fn MediaListScreen() -> Element {
    let nav = use_navigator();
    let media_state = use_media();

    let filters = use_signal(|| MediaListQuery::new());
    let selected_ids = use_signal(|| Vec::<i32>::new());
    let usage_dialog_open = use_signal(|| false);
    let usage_dialog_media = use_signal(|| None::<Media>);

    // Use the enhanced hook that creates handlers for us
    let (list_state, handlers) = use_list_screen_with_handlers(
        Some(ListScreenConfig {
            default_sort_field: "created_at".to_string(),
            default_sort_order: Order::Desc,
        }),
        filters,
    );

    // Effect to load data when filters change
    use_effect({
        let list_state = list_state;
        let mut selected_ids = selected_ids;
        move || {
            let q = filters();
            let _tick = list_state.reload_tick();
            let media_state = media_state;
            selected_ids.set(Vec::new());
            spawn(async move {
                media_state.fetch_list_with_query(q).await;
            });
        }
    });

    let list = media_state.list.read();
    let list_loading = list.is_loading();
    let _list_failed = list.is_failed();

    let (media_items, current_page) = if let Some(p) = &list.data {
        (p.data.clone(), p.page)
    } else {
        (Vec::<Media>::new(), 1)
    };

    let has_data = !media_items.is_empty();

    // Define header columns
    let headers = vec![
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
        HeaderColumn::new("", false, "w-16 py-2 px-3", None), // Thumbnail
        HeaderColumn::new(
            "File Name",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("object_key"),
        ),
        HeaderColumn::new(
            "Type",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("mime_type"),
        ),
        HeaderColumn::new(
            "Size",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("size"),
        ),
        HeaderColumn::new(
            "Dimensions",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Reference",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Usage",
            false,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            None,
        ),
        HeaderColumn::new(
            "Uploaded",
            true,
            "py-2 px-3 text-left font-medium text-xs md:text-sm whitespace-nowrap",
            Some("created_at"),
        ),
        HeaderColumn::new("", false, "w-12 py-2 px-3", None),
    ];

    // Reference type filter handler
    let handle_reference_select = {
        let mut filters = filters;
        move |value: String| {
            let mut q = filters.peek().clone();
            q.set_page(1);
            q.reference_type = match value.as_str() {
                "Post" => Some(MediaReference::Post),
                "Category" => Some(MediaReference::Category),
                "User" => Some(MediaReference::User),
                _ => None,
            };
            filters.set(q);
        }
    };

    rsx! {
        DataTableScreen::<Media> {
            frame: (media_state.list)(),
            header: Some(PageHeaderProps {
                title: "Media".to_string(),
                description: "Manage your media files".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: move |_| { nav.push(Route::MediaUploadScreen {}); },
                        "Upload Media"
                    }
                }),
                class: None,
                embedded: false,
            }),
            headers: Some(headers),
            current_sort_field: Some(list_state.sort_field()),
            on_sort: Some(handlers.handle_sort.clone()),
            error_title: Some("Failed to load media".to_string()),
            error_retry_label: Some("Retry".to_string()),
            on_error_retry: Some(EventHandler::new(move |_| handlers.handle_retry.call(()))),
            toolbar: Some(ListToolbarProps {
                search_value: list_state.search_input(),
                search_placeholder: "Search media by filename or type".to_string(),
                disabled: list_loading,
                on_search_input: handlers.handle_search.clone(),
                status_selected: match &filters.read().reference_type {
                    Some(MediaReference::Post) => "Post".to_string(),
                    Some(MediaReference::Category) => "Category".to_string(),
                    Some(MediaReference::User) => "User".to_string(),
                    None => "All".to_string(),
                },
                on_status_select: EventHandler::new(handle_reference_select),
            }),
            on_prev: move |_| { handlers.handle_prev.call(current_page); },
            on_next: move |_| { handlers.handle_next.call(current_page); },
            // Bulk actions
            below_toolbar: if !selected_ids.read().is_empty() {
                Some(rsx! {
                    div { class: "w-full flex items-center justify-between bg-transparent border border-zinc-200 dark:border-zinc-800 rounded-md px-4 py-3 shadow-sm",
                        span { class: "text-sm text-muted-foreground", "{selected_ids.read().len()} selected" }
                        div { class: "flex items-center gap-2",
                            Button { variant: ButtonVariant::Outline, class: "h-8 text-red-600 border-red-200 dark:border-red-800 dark:hover:bg-red-950/20 hover:bg-red-50",
                                onclick: {
                                    let selected_ids_clone = selected_ids.read().clone();
                                    let mut selected_ids = selected_ids;
                                    move |_| {
                                        for id in &selected_ids_clone {
                                            let id = *id;
                                            spawn(async move {
                                                let media_state = use_media();
                                                media_state.remove(id).await;
                                            });
                                        }
                                        selected_ids.set(Vec::new());
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                })
            } else { None },
            // Table body
            if media_items.is_empty() {
                if list_loading && !has_data {
                    SkeletonTableRows {
                        row_count: 6,
                        cells: vec![
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "w-12 py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Avatar, "w-16 py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Badge, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Badge, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Default, "py-2 px-3"),
                            SkeletonCellConfig::custom(crate::components::UICellType::Action, "py-2 px-3"),
                        ],
                    }
                } else {
                    tr { class: "border-b border-zinc-200 dark:border-zinc-800",
                        td { colspan: "10", class: "py-12 px-4 text-center",
                            ListEmptyState {
                                title: "No media found".to_string(),
                                description: "Try adjusting your search or upload media to get started.".to_string(),
                                clear_label: "Clear search".to_string(),
                                create_label: "Upload your first file".to_string(),
                                on_clear: move |_| { handlers.handle_clear.call(()); },
                                on_create: move |_| { nav.push(Route::MediaUploadScreen {}); },
                            }
                        }
                    }
                }
            } else {
                {media_items.iter().cloned().map(|media| {
                    let media_id = media.id;
                    let filename = media.object_key.split('/').last().unwrap_or("Unknown").to_string();
                    let mime_type = media.mime_type.clone();
                    let file_type_short = if let Some(ext) = &media.extension {
                        ext.to_uppercase()
                    } else {
                        mime_type.split('/').next().unwrap_or("FILE").to_uppercase()
                    };

                    // Add toast notification for delete operations
                    let toast_config = StateFrameToastConfig {
                        loading_title: "Deleting media...".to_string(),
                        success_title: Some("Media deleted successfully".to_string()),
                        error_title: Some("Failed to delete media".to_string()),
                        ..Default::default()
                    };
                    use_state_frame_map_toast(&media_state.remove, media_id, toast_config);

                    // Check if this item is being deleted
                    let is_deleting = media_state.remove.read().get(&media_id).map_or(false, |frame| frame.is_loading());

                    rsx! {
                        tr {
                            key: "{media_id}",
                            class: "relative border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                            // Selection checkbox cell
                            td {
                                class: "py-2 px-3 w-12 text-xs md:text-sm",
                                Checkbox {
                                    checked: selected_ids.read().contains(&media_id),
                                    onchange: Some(EventHandler::new({
                                        let mut selected_ids = selected_ids;
                                        move |checked: bool| {
                                            let mut current = selected_ids.peek().clone();
                                            if checked {
                                                if !current.contains(&media_id) {
                                                    current.push(media_id);
                                                }
                                            } else {
                                                current.retain(|&id| id != media_id);
                                            }
                                            selected_ids.set(current);
                                        }
                                    })),
                                }
                                LoadingOverlay { visible: is_deleting }
                            }
                            // Thumbnail cell
                            td { class: "py-2 px-3 w-16",
                                if is_image(&mime_type) {
                                    img {
                                        src: "{media.file_url}",
                                        alt: "{filename}",
                                        class: "w-12 h-12 object-cover rounded border border-zinc-200 dark:border-zinc-700",
                                    }
                                } else {
                                    div { class: "w-12 h-12 flex items-center justify-center rounded border border-zinc-200 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-800",
                                        span { class: "text-xs text-muted-foreground font-mono", "{file_type_short}" }
                                    }
                                }
                            }
                            // Filename cell
                            td { class: "py-2 px-3 text-xs md:text-sm whitespace-nowrap",
                                span { class: "font-medium leading-none truncate max-w-xs block", "{filename}" }
                            }
                            // Type cell
                            td { class: "py-2 px-3 text-xs md:text-sm",
                                Badge { class: "font-mono text-xs", "{file_type_short}" }
                            }
                            // Size cell
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                                "{format_file_size(media.size)}"
                            }
                            // Dimensions cell
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                                if let (Some(w), Some(h)) = (media.width, media.height) {
                                    span { "{w} × {h}" }
                                } else {
                                    span { "—" }
                                }
                            }
                            // Reference cell
                            td { class: "py-2 px-3 text-xs md:text-sm",
                                if let Some(ref_type) = &media.reference_type {
                                    Badge { class: "text-xs", "{ref_type}" }
                                } else {
                                    span { class: "text-muted-foreground", "—" }
                                }
                            }
                            // Usage cell
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                                span {
                                    class: "font-medium",
                                    "{media.usage_count} refs"
                                }
                            }
                            // Upload date cell
                            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                                "{format_short_date_dt(&media.created_at)}"
                            }
                            // Actions cell
                            td { class: "py-2 px-3 text-xs md:text-sm",
                                DropdownMenu {
                                    DropdownMenuTrigger {
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            class: "h-8 w-8 p-0 bg-transparent hover:bg-muted/50",
                                            div { class: "w-4 h-4", Icon { icon: LdEllipsis {} } }
                                        }
                                    }
                                    DropdownMenuContent { class: "bg-background border-zinc-200 dark:border-zinc-800",
                                        DropdownMenuItem {
                                            onclick: {
                                                let url = media.file_url.clone();
                                                move |_| {
                                                    // Open in new tab
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.open_with_url_and_target(&url, "_blank");
                                                    }
                                                }
                                            },
                                            "View Full Size"
                                        }
                                        DropdownMenuItem {
                                            onclick: {
                                                let media_clone = media.clone();
                                                let mut usage_dialog_open = usage_dialog_open;
                                                let mut usage_dialog_media = usage_dialog_media;
                                                move |_| {
                                                    usage_dialog_open.set(true);
                                                    usage_dialog_media.set(Some(media_clone.clone()));
                                                }
                                            },
                                            "View Usage"
                                        }
                                        DropdownMenuItem {
                                            onclick: {
                                                let url = media.file_url.clone();
                                                move |_| {
                                                    // Copy to clipboard
                                                    if let Some(window) = web_sys::window() {
                                                        let navigator = window.navigator();
                                                        let clipboard = navigator.clipboard();
                                                        let _ = clipboard.write_text(&url);
                                                    }
                                                }
                                            },
                                            "Copy URL"
                                        }
                                        DropdownMenuItem { class: "text-red-600", onclick: move |_| {
                                                let id = media_id;
                                                spawn({  async move {
                                                    let media_state = use_media();
                                                    media_state.remove(id).await;
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

        // Media Usage Dialog
        if let Some(media) = &*usage_dialog_media.read() {
            MediaUsageDialog {
                is_open: usage_dialog_open,
                media: media.clone(),
            }
        }
    }
}
