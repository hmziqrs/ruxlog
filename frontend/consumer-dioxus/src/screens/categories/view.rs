use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::{use_categories, use_post, PostListQuery};
use crate::router::Route;
use crate::components::{PostCard, PostsLoadingSkeleton};
use hmziq_dioxus_free_icons::icons::ld_icons::LdArrowLeft;
use hmziq_dioxus_free_icons::Icon;

#[component]
pub fn CategoryDetailScreen(slug: String) -> Element {
    let categories_store = use_categories();
    let posts_store = use_post();
    let nav = use_navigator();

    // Fetch categories if not loaded
    use_effect(move || {
        let categories = categories_store;
        spawn(async move {
            categories.list().await;
        });
    });

    // Find category by slug
    let category = use_memo(move || {
        let categories_read = categories_store.list.read();
        if let Some(list) = &(*categories_read).data {
            list.data.iter().find(|c| c.slug == slug).cloned()
        } else {
            None
        }
    });

    // Fetch posts filtered by category when category is found
    use_effect(move || {
        if let Some(c) = category() {
            let posts = posts_store;
            let category_id = c.id;
            spawn(async move {
                let query = PostListQuery {
                    page: Some(1),
                    category_id: Some(category_id),
                    ..Default::default()
                };
                posts.list_with_query(query).await;
            });
        }
    });

    let posts_frame = posts_store.list.read();
    let categories_frame = categories_store.list.read();

    let on_post_click = move |post_id: i32| {
        nav.push(Route::PostViewScreen { id: post_id });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                if (*categories_frame).is_loading() {
                    // Loading skeleton
                    div { class: "mb-12",
                        div { class: "h-8 w-48 bg-muted rounded animate-pulse mb-4" }
                        div { class: "h-6 w-64 bg-muted rounded animate-pulse" }
                    }
                } else if let Some(c) = category() {
                    // Header
                    div { class: "mb-12",
                        // Back button
                        button {
                            class: "flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors mb-6 group",
                            onclick: move |_| {
                                nav.push(Route::CategoriesScreen {});
                            },
                            Icon {
                                icon: LdArrowLeft,
                                class: "w-4 h-4 transition-transform group-hover:-translate-x-1",
                            }
                            span { class: "text-sm", "Back to all categories" }
                        }

                        // Category header with cover/logo
                        div {
                            class: "relative rounded-lg overflow-hidden mb-6",

                            // Cover image or gradient background
                            if let Some(cover) = &c.cover {
                                div {
                                    class: "h-40 bg-cover bg-center",
                                    style: "background-image: url('{cover.file_url}');",
                                }
                            } else {
                                div {
                                    class: "h-40",
                                    style: "background: linear-gradient(135deg, {c.color} 0%, {c.color}dd 100%);",
                                }
                            }

                            // Overlay content
                            div { class: "absolute inset-0 bg-gradient-to-t from-background/90 to-transparent flex items-end p-6",
                                div { class: "flex items-center gap-4",
                                    if let Some(logo) = &c.logo {
                                        img {
                                            src: "{logo.file_url}",
                                            alt: "{c.name}",
                                            class: "w-16 h-16 rounded-lg border-2 border-background shadow-lg object-cover",
                                        }
                                    }

                                    div {
                                        h1 { class: "text-3xl md:text-4xl font-bold tracking-tight text-foreground",
                                            "{c.name}"
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(description) = &c.description {
                            p { class: "text-muted-foreground text-lg",
                                "{description}"
                            }
                        }
                    }

                    // Posts section
                    if (*posts_frame).is_loading() {
                        PostsLoadingSkeleton {}
                    } else if (*posts_frame).is_failed() {
                        div { class: "flex items-center justify-center py-20",
                            div { class: "max-w-md w-full",
                                ErrorDetails {
                                    error: (*posts_frame).error.clone(),
                                    variant: ErrorDetailsVariant::Collapsed,
                                }
                            }
                        }
                    } else if let Some(data) = &(*posts_frame).data {
                        if data.data.is_empty() {
                            div { class: "flex flex-col items-center justify-center py-20 text-center",
                                p { class: "text-muted-foreground text-lg mb-4",
                                    "No posts found in this category"
                                }
                                button {
                                    class: "text-sm text-primary hover:underline",
                                    onclick: move |_| {
                                        nav.push(Route::HomeScreen {});
                                    },
                                    "Browse all posts"
                                }
                            }
                        } else {
                            div { class: "space-y-6",
                                div { class: "text-sm text-muted-foreground mb-6",
                                    {format!("{} {} found", data.data.len(), if data.data.len() == 1 { "post" } else { "posts" })}
                                }

                                div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                                    for post in data.data.iter() {
                                        PostCard {
                                            key: "{post.id}",
                                            post: post.clone(),
                                            on_click: on_post_click,
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Category not found
                    div { class: "flex flex-col items-center justify-center py-20 text-center",
                        p { class: "text-muted-foreground text-lg mb-4",
                            "Category not found"
                        }
                        button {
                            class: "text-sm text-primary hover:underline",
                            onclick: move |_| {
                                nav.push(Route::CategoriesScreen {});
                            },
                            "Browse all categories"
                        }
                    }
                }
            }
        }
    }
}
