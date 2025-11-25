mod view;

pub use view::*;

use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::use_categories;
use crate::router::Route;

#[component]
pub fn CategoriesScreen() -> Element {
    let categories_store = use_categories();
    let nav = use_navigator();

    use_effect(move || {
        let categories = categories_store;
        spawn(async move {
            categories.list().await;
        });
    });

    let categories_frame = categories_store.list.read();

    let on_category_click = move |slug: String| {
        nav.push(Route::CategoryDetailScreen { slug });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                // Header
                div { class: "mb-12",
                    h1 { class: "text-3xl md:text-4xl font-bold tracking-tight mb-3",
                        "Browse Categories"
                    }
                    p { class: "text-muted-foreground text-lg",
                        "Explore articles by categories"
                    }
                }

                if (*categories_frame).is_loading() {
                    // Loading skeleton
                    div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                        for _ in 0..6 {
                            div { class: "h-48 bg-muted rounded-lg animate-pulse" }
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
                        div { class: "flex flex-col items-center justify-center py-20 text-center",
                            p { class: "text-muted-foreground text-lg", "No categories found" }
                        }
                    } else {
                        div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for category in data.data.iter() {
                                {
                                    let category_slug = category.slug.clone();
                                    let category_color = category.color.clone();
                                    let category_name = category.name.clone();
                                    let category_description = category.description.clone();
                                    let category_cover = category.cover.clone();
                                    let category_logo = category.logo.clone();
                                    let category_id = category.id;
                                    rsx! {
                                        button {
                                            key: "{category_id}",
                                            class: "group relative overflow-hidden rounded-lg border border-border bg-card hover:bg-accent transition-all text-left h-48",
                                            onclick: move |_| {
                                                on_category_click(category_slug.clone());
                                            },

                                    // Cover image background
                                    if let Some(cover) = &category_cover {
                                        div {
                                            class: "absolute inset-0 bg-cover bg-center opacity-20 group-hover:opacity-30 transition-opacity",
                                            style: "background-image: url('{cover.file_url}');",
                                        }
                                    } else {
                                        div {
                                            class: "absolute inset-0 opacity-10",
                                            style: "background: linear-gradient(135deg, {category_color} 0%, transparent 100%);",
                                        }
                                    }

                                    // Content
                                    div { class: "relative p-6 h-full flex flex-col justify-between",
                                        // Logo and title
                                        div {
                                            if let Some(logo) = &category_logo {
                                                img {
                                                    src: "{logo.file_url}",
                                                    alt: "{category_name}",
                                                    class: "w-12 h-12 rounded-lg mb-4 object-cover",
                                                }
                                            }

                                            h3 { class: "text-xl font-bold text-foreground group-hover:text-primary transition-colors mb-2",
                                                "{category_name}"
                                            }

                                            if let Some(description) = &category_description {
                                                p { class: "text-sm text-muted-foreground line-clamp-2",
                                                    "{description}"
                                                }
                                            }
                                        }

                                        // Color indicator
                                        div {
                                            class: "flex items-center gap-2",
                                            div {
                                                class: "w-3 h-3 rounded-full",
                                                style: "background-color: {category_color};",
                                            }
                                            span { class: "text-xs text-muted-foreground",
                                                "View articles"
                                            }
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
