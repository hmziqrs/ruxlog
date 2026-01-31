use crate::components::{
    BannerPlaceholder, FeaturedPostCard, PostCard, PostsEmptyState, PostsLoadingSkeleton,
};
use crate::router::Route;
use crate::seo::{use_static_seo, website_schema, SeoHead, StructuredData};
use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::use_post;

#[component]
pub fn HomeScreen() -> Element {
    let posts_store = use_post();
    let nav = use_navigator();

    // Generate SEO metadata for homepage
    let seo_metadata = use_static_seo("home");

    use_effect(move || {
        let posts = posts_store;
        spawn(async move {
            posts.list_published().await;
        });
    });

    let posts_frame = posts_store.list.read();

    let on_post_click = move |post_slug: String| {
        nav.push(Route::PostViewScreen { slug: post_slug });
    };

    rsx! {
        // Inject SEO tags
        SeoHead { metadata: seo_metadata }

        // Inject structured data
        StructuredData { json_ld: website_schema() }

        div { class: "min-h-screen bg-background",
            div { class: "h-4" }
            BannerPlaceholder {}

            div { class: "container mx-auto px-4 py-4 max-w-6xl",
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
                        PostsEmptyState {}
                    } else {
                        div { class: "space-y-10",
                            // Featured post (hero card)
                            if let Some(featured) = data.data.first() {
                                FeaturedPostCard {
                                    post: featured.clone(),
                                    on_click: on_post_click,
                                }
                            }

                            // Posts grid
                            if data.data.len() > 1 {
                                div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                                    for post in data.data.iter().skip(1) {
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
                    div { class: "flex items-center justify-center py-20",
                        div { "No content available" }
                    }
                }
            }
            div { class: "h-8" }

        }
    }
}
