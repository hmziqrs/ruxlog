mod view;

pub use view::*;

use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::use_tag;
use crate::router::Route;
use crate::components::TagCard;

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
        div { class: "min-h-screen bg-background",
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
