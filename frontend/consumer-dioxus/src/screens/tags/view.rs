use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::{use_post, use_tag, PostListQuery};
use crate::router::Route;
use crate::components::{PostCard, PostsLoadingSkeleton};
use hmziq_dioxus_free_icons::icons::ld_icons::LdArrowLeft;
use hmziq_dioxus_free_icons::Icon;

#[component]
pub fn TagDetailScreen(slug: String) -> Element {
    let tags_store = use_tag();
    let posts_store = use_post();
    let nav = use_navigator();

    // Fetch tags if not loaded
    use_effect(move || {
        let tags = tags_store;
        spawn(async move {
            tags.list().await;
        });
    });

    // Find tag by slug
    let tag = use_memo(move || {
        let tags_read = tags_store.list.read();
        if let Some(list) = &(*tags_read).data {
            list.data.iter().find(|t| t.slug == slug).cloned()
        } else {
            None
        }
    });

    // Fetch posts filtered by tag when tag is found
    use_effect(move || {
        if let Some(t) = tag() {
            let posts = posts_store;
            let tag_id = t.id;
            spawn(async move {
                let query = PostListQuery {
                    page: Some(1),
                    tag_ids: Some(vec![tag_id]),
                    ..Default::default()
                };
                posts.list_with_query(query).await;
            });
        }
    });

    let posts_frame = posts_store.list.read();
    let tags_frame = tags_store.list.read();

    let on_post_click = move |post_id: i32| {
        nav.push(Route::PostViewScreen { id: post_id });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                if (*tags_frame).is_loading() {
                    // Loading skeleton
                    div { class: "mb-12",
                        div { class: "h-8 w-48 bg-muted rounded animate-pulse mb-4" }
                        div { class: "h-6 w-64 bg-muted rounded animate-pulse" }
                    }
                } else if let Some(t) = tag() {
                    // Header
                    div { class: "mb-12",
                        // Back button
                        button {
                            class: "flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors mb-6 group",
                            onclick: move |_| {
                                nav.push(Route::TagsScreen {});
                            },
                            Icon {
                                icon: LdArrowLeft,
                                class: "w-4 h-4 transition-transform group-hover:-translate-x-1",
                            }
                            span { class: "text-sm", "Back to all tags" }
                        }

                        div { class: "flex items-center gap-4 mb-4",
                            // Tag color indicator
                            div {
                                class: "w-4 h-4 rounded-full flex-shrink-0",
                                style: "background-color: {t.color};",
                            }
                            h1 { class: "text-3xl md:text-4xl font-bold tracking-tight",
                                "{t.name}"
                            }
                        }

                        if let Some(description) = &t.description {
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
                                    "No posts found with this tag"
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
                    // Tag not found
                    div { class: "flex flex-col items-center justify-center py-20 text-center",
                        p { class: "text-muted-foreground text-lg mb-4",
                            "Tag not found"
                        }
                        button {
                            class: "text-sm text-primary hover:underline",
                            onclick: move |_| {
                                nav.push(Route::TagsScreen {});
                            },
                            "Browse all tags"
                        }
                    }
                }
            }
        }
    }
}
