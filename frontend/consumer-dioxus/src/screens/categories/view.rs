use crate::components::{PostCard, PostsLoadingSkeleton};
use crate::router::Route;
use crate::seo::{breadcrumb_schema, use_category_seo, SeoHead, StructuredData};
use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::{use_categories, use_post, PostListQuery};

#[component]
pub fn CategoryDetailScreen(slug: String) -> Element {
    let categories_store = use_categories();
    let posts_store = use_post();
    let nav = use_navigator();

    // Generate SEO metadata
    let seo_metadata = use_category_seo(slug.clone());

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
                posts.list_published_with_query(query).await;
            });
        }
    });

    let posts_frame = posts_store.list.read();

    let on_post_click = move |post_slug: String| {
        nav.push(Route::PostViewScreen { slug: post_slug });
    };

    rsx! {
        // Inject SEO tags if metadata is available
        if let Some(metadata) = seo_metadata() {
            SeoHead { metadata }
        }

        // Inject breadcrumb structured data if category is loaded
        if let Some(cat) = category() {
            StructuredData {
                json_ld: breadcrumb_schema(vec![
                    ("Home", "/"),
                    ("Categories", "/categories"),
                    (&cat.name, &format!("/categories/{}", cat.slug))
                ])
            }
        }

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
