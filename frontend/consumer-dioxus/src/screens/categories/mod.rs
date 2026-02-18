mod view;

pub use view::*;

use crate::components::CategoryCard;
use crate::router::Route;
use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};

#[cfg(feature = "demo-static-content")]
use crate::demo_content;
#[cfg(not(feature = "demo-static-content"))]
use ruxlog_shared::store::use_categories;

#[cfg(not(feature = "demo-static-content"))]
#[component]
pub fn CategoriesScreen() -> Element {
    let categories_store = use_categories();
    let nav = use_navigator();

    use_effect(move || {
        let categories = categories_store;
        spawn(async move {
            categories.list_all().await;
        });
    });

    let categories_frame = categories_store.list.read();

    let on_category_click = move |slug: String| {
        nav.push(Route::CategoryDetailScreen { slug });
    };

    rsx! {
        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                if (*categories_frame).is_loading() {
                    div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                        for _ in 0..6 {
                            div { class: "h-64 bg-muted rounded-lg animate-pulse" }
                        }
                    }
                } else if (*categories_frame).is_failed() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "max-w-md w-full",
                            ErrorDetails {
                                error: (*categories_frame).error.clone(),
                                variant: ErrorDetailsVariant::Collapsed,
                            }
                        }
                    }
                } else if let Some(data) = &(*categories_frame).data {
                    if data.data.is_empty() {
                        div { class: "flex items-center justify-center py-20",
                            div { "No categories found" }
                        }
                    } else {
                        div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for category in data.data.iter() {
                                CategoryCard {
                                    key: "{category.id}",
                                    category: category.clone(),
                                    on_click: on_category_click,
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
pub fn CategoriesScreen() -> Element {
    let nav = use_navigator();

    let categories = demo_content::content().categories().to_vec();

    let on_category_click = move |slug: String| {
        nav.push(Route::CategoryDetailScreen { slug });
    };

    let content = if categories.is_empty() {
        rsx! {
            div { class: "flex items-center justify-center py-20",
                div { "No categories found" }
            }
        }
    } else {
        rsx! {
            div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                for category in categories.iter() {
                    CategoryCard {
                        key: "{category.id}",
                        category: category.clone(),
                        on_click: on_category_click,
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
