use crate::components::{PostCard, PostsLoadingSkeleton};
use crate::router::Route;
use crate::seo::{breadcrumb_schema, SeoHead, SeoMetadataBuilder, StructuredData};
use crate::server_fns::{fetch_category_by_slug, fetch_posts_by_category};
use dioxus::prelude::*;
use oxstore::AppError;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};

#[component]
pub fn CategoryDetailScreen(slug: String) -> Element {
    let nav = use_navigator();

    // SSR: Fetch category by slug
    let category_result = use_server_future(move || {
        let slug = slug.clone();
        async move { fetch_category_by_slug(slug).await }
    })?;

    // Get category for dependent query
    let category = match category_result() {
        Some(Ok(c)) => c,
        Some(Err(e)) => {
            return rsx! {
                div { class: "min-h-screen flex items-center justify-center",
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

    let Some(category) = category else {
        return rsx! {
            div { class: "min-h-screen flex items-center justify-center",
                div { class: "text-center",
                    h1 { class: "text-2xl font-bold mb-4", "Category not found" }
                    button {
                        class: "text-primary hover:underline",
                        onclick: move |_| { nav.push(Route::HomeScreen {}); },
                        "Back to home"
                    }
                }
            }
        };
    };

    // SSR: Fetch posts by category
    let category_id = category.id;
    let posts_result =
        use_server_future(move || async move { fetch_posts_by_category(category_id).await })?;

    let on_post_click = move |post_slug: String| {
        nav.push(Route::PostViewScreen { slug: post_slug });
    };

    let cat_name = category.name.clone();
    let cat_slug = category.slug.clone();

    rsx! {
        SeoHead {
            metadata: SeoMetadataBuilder::new()
                .title(&cat_name)
                .description(&format!("Browse all posts in the {} category", cat_name))
                .canonical(&format!("/categories/{}", cat_slug))
                .build()
        }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Categories", "/categories"),
                (&cat_name, &format!("/categories/{}", cat_slug))
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                h1 { class: "text-3xl font-bold mb-8", "{cat_name}" }

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
                    None => rsx! { PostsLoadingSkeleton {} },
                }
            }
        }
    }
}
