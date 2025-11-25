use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::{use_post, use_tag, PostListQuery};
use crate::router::Route;
use crate::components::{PostCard, PostsLoadingSkeleton};

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

    let on_post_click = move |post_id: i32| {
        nav.push(Route::PostViewScreen { id: post_id });
    };

    rsx! {
        div { class: "min-h-screen bg-background",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
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
                        div { class: "flex items-center justify-center py-20",
                            div { "No posts found" }
                        }
                    } else {
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
                } else {
                    div { class: "flex items-center justify-center py-20",
                        div { "No content available" }
                    }
                }
            }
        }
    }
}
