use crate::components::PostCard;
use crate::router::Route;
use crate::seo::{breadcrumb_schema, SeoHead, SeoMetadataBuilder, StructuredData};
use crate::server_fns::{fetch_posts_by_tag, fetch_tag_by_slug};
use dioxus::prelude::*;
use oxstore::AppError;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};

#[component]
pub fn TagDetailScreen(slug: String) -> Element {
    let nav = use_navigator();

    // SSR: Fetch tag by slug
    let tag_result = use_server_future(move || {
        let slug = slug.clone();
        async move { fetch_tag_by_slug(slug).await }
    })?;

    // Get tag for dependent query
    let tag = match tag_result() {
        Some(Ok(t)) => t,
        Some(Err(e)) => {
            return rsx! {
                div { class: "min-h-screen bg-background flex items-center justify-center",
                    div { class: "max-w-md w-full",
                        ErrorDetails {
                            error: Some(AppError::Other { message: e.to_string() }),
                            variant: ErrorDetailsVariant::Collapsed,
                        }
                    }
                }
            };
        }
        None => return rsx! { div { "Loading..." } },
    };

    let Some(tag) = tag else {
        return rsx! {
            div { class: "min-h-screen bg-background flex items-center justify-center",
                div { class: "text-center",
                    h1 { class: "text-2xl font-bold mb-4", "Tag not found" }
                    button {
                        class: "text-primary hover:underline",
                        onclick: move |_| { nav.push(Route::HomeScreen {}); },
                        "Back to home"
                    }
                }
            }
        };
    };

    // SSR: Fetch posts by tag
    let tag_id = tag.id;
    let posts_result = use_server_future(move || async move { fetch_posts_by_tag(tag_id).await })?;

    let on_post_click = move |post_slug: String| {
        nav.push(Route::PostViewScreen { slug: post_slug });
    };

    let tag_name = tag.name.clone();
    let tag_slug = tag.slug.clone();

    rsx! {
        // Inject SEO tags
        SeoHead {
            metadata: SeoMetadataBuilder::new()
                .title(&tag_name)
                .description(&format!("Browse all posts tagged with {}", tag_name))
                .canonical(&format!("/tags/{}", tag_slug))
                .build()
        }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Tags", "/tags"),
                (&tag_name, &format!("/tags/{}", tag_slug))
            ])
        }

        div { class: "min-h-screen bg-background",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                // Tag header
                h1 { class: "text-3xl font-bold mb-8", "Posts tagged: {tag_name}" }

                match posts_result() {
                    Some(Ok(data)) => {
                        if data.data.is_empty() {
                            rsx! {
                                div { class: "flex items-center justify-center py-20",
                                    div { "No posts found" }
                                }
                            }
                        } else {
                            rsx! {
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
                    Some(Err(e)) => rsx! {
                        div { class: "flex items-center justify-center py-20",
                            div { class: "max-w-md w-full",
                                ErrorDetails {
                                    error: Some(AppError::Other { message: e.to_string() }),
                                    variant: ErrorDetailsVariant::Collapsed,
                                }
                            }
                        }
                    },
                    None => rsx! { div { "Loading posts..." } },
                }
            }
        }
    }
}
