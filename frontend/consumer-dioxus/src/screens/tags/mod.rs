mod view;

pub use view::*;

use crate::components::TagCard;
use crate::router::Route;
use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};

#[cfg(feature = "demo-static-content")]
use crate::demo_content;
#[cfg(not(feature = "demo-static-content"))]
use ruxlog_shared::store::use_tag;

#[cfg(not(feature = "demo-static-content"))]
#[component]
pub fn TagsScreen() -> Element {
    let tags_store = use_tag();
    let nav = use_navigator();

    use_effect(move || {
        let tags = tags_store;
        spawn(async move {
            tags.list_all().await;
        });
    });

    let tags_frame = tags_store.list.read();

    let on_tag_click = move |slug: String| {
        nav.push(Route::TagDetailScreen { slug });
    };

    rsx! {
        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                if (*tags_frame).is_loading() {
                    div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                        for _ in 0..8 {
                            div { class: "h-32 bg-muted rounded-lg animate-pulse" }
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
                        div { class: "flex items-center justify-center py-20",
                            div { "No tags found" }
                        }
                    } else {
                        div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                            for tag in data.data.iter() {
                                TagCard {
                                    key: "{tag.id}",
                                    tag: tag.clone(),
                                    on_click: on_tag_click,
                                }
                            }
                        }
                    }
                } else {
                    div { class: "flex items-center justify-center py-20",
                        div { "No content available" }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "demo-static-content")]
#[component]
pub fn TagsScreen() -> Element {
    let nav = use_navigator();

    let tags = demo_content::content().tags().to_vec();

    let on_tag_click = move |slug: String| {
        nav.push(Route::TagDetailScreen { slug });
    };

    let content = if tags.is_empty() {
        rsx! {
            div { class: "flex items-center justify-center py-20",
                div { "No tags found" }
            }
        }
    } else {
        rsx! {
            div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                for tag in tags.iter() {
                    TagCard {
                        key: "{tag.id}",
                        tag: tag.clone(),
                        on_click: on_tag_click,
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                {content}
            }
        }
    }
}
