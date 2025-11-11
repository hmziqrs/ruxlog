use crate::store::{use_media, Media};
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Badge, Button};
use crate::utils::dates::format_short_date_dt;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

#[component]
pub fn MediaUsageDialog(is_open: Signal<bool>, media: Media) -> Element {
    let media_state = use_media();

    use_effect({
        let is_open = is_open;
        let media_id = media.id;
        let media_state = media_state;
        move || {
            if *is_open.read() {
                let media_id = media_id;
                let media_state = media_state;
                spawn(async move {
                    media_state.usage_details(media_id).await;
                });
            }
        }
    });

    let handle_close = move |_| {
        is_open.set(false);
    };

    if !*is_open.read() {
        return rsx! {};
    }

    let usage_map = media_state.usage_details.read();
    let usage_frame = usage_map.get(&media.id);
    let is_loading = usage_frame.map(|f| f.is_loading()).unwrap_or(false);
    let details = usage_frame.and_then(|f| f.data.clone());

    rsx! {
        AppPortal {
            z_index: "1100",
            div {
                class: "fixed inset-0 bg-black/50",
                onclick: handle_close,
            }

            div {
                class: "fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%] w-full max-w-4xl max-h-[90vh] overflow-hidden",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "bg-background rounded-lg border shadow-lg flex flex-col max-h-[90vh]",
                    div { class: "flex items-center justify-between p-6 border-b",
                        h2 { class: "text-lg font-semibold", "Media Usage" }
                        button {
                            onclick: handle_close,
                            class: "rounded-xs opacity-70 hover:opacity-100 transition-opacity",
                            Icon { icon: LdX, width: 20, height: 20 }
                        }
                    }

                    div { class: "overflow-y-auto flex-1",
                        if is_loading {
                            div { class: "p-6 text-center text-muted-foreground",
                                "Loading usage details..."
                            }
                        } else if let Some(details) = details {
                            div { class: "p-6 space-y-6",
                                // Media info
                                div { class: "bg-muted/30 rounded-lg p-4",
                                    div { class: "flex items-center gap-4",
                                        img {
                                            src: "{details.media.file_url}",
                                            alt: "Media thumbnail",
                                            class: "w-20 h-20 object-cover rounded border",
                                        }
                                        div { class: "flex-1 min-w-0",
                                            h3 { class: "font-medium truncate", "Media Details" }
                                            div { class: "text-sm text-muted-foreground space-y-1",
                                                p { "Object Key: {details.media.object_key}" }
                                                p { "Type: {details.media.mime_type}" }
                                                p { "Size: {details.media.size} bytes" }
                                            }
                                        }
                                    }
                                }

                                // Posts section
                                if !details.posts.is_empty() {
                                    div { class: "space-y-3",
                                        h3 { class: "text-base font-semibold flex items-center gap-2",
                                            Badge { class: "text-xs", "Posts" }
                                            span { class: "text-muted-foreground text-sm font-normal", "({details.posts.len()})" }
                                        }
                                        div { class: "space-y-2",
                                            {details.posts.iter().map(|post_usage| {
                                                let post = &post_usage.post;
                                                rsx! {
                                                    div { class: "border rounded-lg p-4 hover:bg-muted/30 transition-colors",
                                                        div { class: "flex items-center justify-between mb-2",
                                                            a {
                                                                href: "#",
                                                                class: "font-medium hover:underline",
                                                                onclick: move |e| {
                                                                    e.prevent_default();
                                                                },
                                                                "{post.title}"
                                                            }
                                                            span { class: "text-xs text-muted-foreground", "{format_short_date_dt(&post_usage.created_at)}" }
                                                        }
                                                        div { class: "text-sm text-muted-foreground",
                                                            span { "Field: " }
                                                            code { class: "text-xs bg-muted px-1.5 py-0.5 rounded", "{post_usage.field_name}" }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }

                                // Categories section
                                if !details.categories.is_empty() {
                                    div { class: "space-y-3",
                                        h3 { class: "text-base font-semibold flex items-center gap-2",
                                            Badge { class: "text-xs", "Categories" }
                                            span { class: "text-muted-foreground text-sm font-normal", "({details.categories.len()})" }
                                        }
                                        div { class: "space-y-2",
                                            {details.categories.iter().map(|cat_usage| {
                                                let category = &cat_usage.category;
                                                rsx! {
                                                    div { class: "border rounded-lg p-4 hover:bg-muted/30 transition-colors",
                                                        div { class: "flex items-center justify-between mb-2",
                                                            a {
                                                                href: "#",
                                                                class: "font-medium hover:underline",
                                                                onclick: move |e| {
                                                                    e.prevent_default();
                                                                },
                                                                "{category.name}"
                                                            }
                                                            span { class: "text-xs text-muted-foreground", "{format_short_date_dt(&cat_usage.created_at)}" }
                                                        }
                                                        div { class: "text-sm text-muted-foreground",
                                                            span { "Field: " }
                                                            code { class: "text-xs bg-muted px-1.5 py-0.5 rounded", "{cat_usage.field_name}" }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }

                                // Users section
                                if !details.users.is_empty() {
                                    div { class: "space-y-3",
                                        h3 { class: "text-base font-semibold flex items-center gap-2",
                                            Badge { class: "text-xs", "Users" }
                                            span { class: "text-muted-foreground text-sm font-normal", "({details.users.len()})" }
                                        }
                                        div { class: "space-y-2",
                                            {details.users.iter().map(|user_usage| {
                                                let user = &user_usage.user;
                                                rsx! {
                                                    div { class: "border rounded-lg p-4 hover:bg-muted/30 transition-colors",
                                                        div { class: "flex items-center justify-between mb-2",
                                                            a {
                                                                href: "#",
                                                                class: "font-medium hover:underline",
                                                                onclick: move |e| {
                                                                    e.prevent_default();
                                                                },
                                                                "{user.name}"
                                                            }
                                                            span { class: "text-xs text-muted-foreground", "{format_short_date_dt(&user_usage.created_at)}" }
                                                        }
                                                        div { class: "text-sm text-muted-foreground",
                                                            span { "Field: " }
                                                            code { class: "text-xs bg-muted px-1.5 py-0.5 rounded", "{user_usage.field_name}" }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }

                                // Empty state
                                if details.posts.is_empty() && details.categories.is_empty() && details.users.is_empty() {
                                    div { class: "text-center py-12 text-muted-foreground",
                                        "This media is not currently used anywhere."
                                    }
                                }
                            }
                        } else {
                            div { class: "p-6 text-center text-muted-foreground",
                                "Failed to load usage details."
                            }
                        }
                    }

                    div { class: "p-6 border-t flex justify-end",
                        Button {
                            onclick: handle_close,
                            "Close"
                        }
                    }
                }
            }
        }
    }
}
