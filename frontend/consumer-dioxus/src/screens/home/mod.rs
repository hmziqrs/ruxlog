use crate::components::{
    BannerPlaceholder, FeaturedPostCard, PostCard, PostsEmptyState, PostsLoadingSkeleton,
};
#[cfg(feature = "demo-static-content")]
use crate::demo_content;
use crate::router::Route;
use crate::seo::{use_static_seo, website_schema, SeoHead, StructuredData};
use crate::server_fns::fetch_posts;
use dioxus::prelude::*;
use oxstore::AppError;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};

#[component]
pub fn HomeScreen() -> Element {
    let nav = use_navigator();

    // Generate SEO metadata for homepage
    let seo_metadata = use_static_seo("home");

    #[cfg(not(feature = "demo-static-content"))]
    let posts_result = use_server_future(|| fetch_posts())?;

    #[cfg(not(feature = "demo-static-content"))]
    let posts_state = posts_result();
    #[cfg(feature = "demo-static-content")]
    let posts_state = Some(Ok::<_, ServerFnError>(demo_content::paginated(
        demo_content::content().posts(),
    )));

    let on_post_click = move |post_slug: String| {
        nav.push(Route::PostViewScreen { slug: post_slug });
    };

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData { json_ld: website_schema() }

        div { class: "min-h-screen",
            div { class: "h-4" }
            BannerPlaceholder {}

            div { class: "container mx-auto px-4 py-4 max-w-6xl",
                match posts_state {
                    Some(Ok(data)) => {
                        if data.data.is_empty() {
                            rsx! { PostsEmptyState {} }
                        } else {
                            rsx! {
                                div { class: "space-y-10",
                                    if let Some(featured) = data.data.first() {
                                        FeaturedPostCard {
                                            post: featured.clone(),
                                            on_click: on_post_click,
                                        }
                                    }

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
            div { class: "h-8" }
        }
    }
}
