mod view;

pub use view::*;

use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::use_tag;
use crate::router::Route;

#[component]
pub fn TagsScreen() -> Element {
    let tags_store = use_tag();
    let nav = use_navigator();

    use_effect(move || {
        let tags = tags_store;
        spawn(async move {
            tags.list().await;
        });
    });

    let tags_frame = tags_store.list.read();

    let on_tag_click = move |slug: String| {
        nav.push(Route::TagDetailScreen { slug });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                // Header
                div { class: "mb-12",
                    h1 { class: "text-3xl md:text-4xl font-bold tracking-tight mb-3",
                        "Browse Tags"
                    }
                    p { class: "text-muted-foreground text-lg",
                        "Explore articles by tags"
                    }
                }

                if (*tags_frame).is_loading() {
                    // Loading skeleton
                    div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4",
                        for _ in 0..8 {
                            div { class: "h-24 bg-muted rounded-lg animate-pulse" }
                        }
                    }
                } else if (*tags_frame).is_failed() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "max-w-md w-full",
                            ErrorDetails {
                                error: (*tags_frame).error.clone(),
                                variant: ErrorDetailsVariant::Collapsed,
                            }
                        }
                    }
                } else if let Some(data) = &(*tags_frame).data {
                    if data.data.is_empty() {
                        div { class: "flex flex-col items-center justify-center py-20 text-center",
                            p { class: "text-muted-foreground text-lg", "No tags found" }
                        }
                    } else {
                        div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4",
                            for tag in data.data.iter() {
                                {
                                    let tag_slug = tag.slug.clone();
                                    rsx! {
                                        button {
                                            key: "{tag.id}",
                                            class: "group relative p-6 rounded-lg border border-border bg-card hover:bg-accent transition-colors text-left",
                                            onclick: move |_| {
                                                on_tag_click(tag_slug.clone());
                                            },

                                    // Tag color indicator
                                    div {
                                        class: "absolute top-3 right-3 w-3 h-3 rounded-full",
                                        style: "background-color: {tag.color};",
                                    }

                                    // Tag name
                                    div { class: "font-semibold text-foreground mb-2 group-hover:text-primary transition-colors",
                                        "{tag.name}"
                                    }

                                    // Tag description
                                    if let Some(description) = &tag.description {
                                        p { class: "text-sm text-muted-foreground line-clamp-2",
                                            "{description}"
                                        }
                                    }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "text-muted-foreground", "No content available" }
                    }
                }
            }
        }
    }
}
