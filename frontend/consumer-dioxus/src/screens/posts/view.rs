use crate::components::{estimate_reading_time, format_date, ActionBar, BannerPlaceholder};
#[cfg(feature = "demo-static-content")]
use crate::demo_content;
use crate::seo::{
    article_schema, breadcrumb_schema, ArticleMetadata, SeoHead, SeoImage, SeoMetadataBuilder,
    StructuredData,
};
#[cfg(not(feature = "demo-static-content"))]
use crate::server_fns::fetch_post_by_slug;
use crate::utils::editorjs::render_editorjs_content;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdArrowLeft, LdCalendar, LdClock};
use hmziq_dioxus_free_icons::Icon;
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::store::Post;

#[cfg(feature = "engagement")]
use crate::components::EngagementBar;
#[cfg(feature = "engagement")]
use ruxlog_shared::store::{use_auth, use_likes};

#[cfg(feature = "comments")]
use crate::components::CommentsSection;

#[cfg(feature = "analytics")]
use crate::analytics::{tracker, use_page_timer, use_scroll_depth};

/// Generate SEO metadata for a post
fn generate_post_seo(post: &Post) -> crate::seo::SeoMetadata {
    let image = post.featured_image.as_ref().map(|img| SeoImage {
        url: img.file_url.clone(),
        alt: post.title.clone(),
        width: None,
        height: None,
    });

    let article = ArticleMetadata {
        published_time: post.published_at.unwrap_or(post.created_at),
        modified_time: post.updated_at,
        author: post.author.name.clone(),
        section: Some(post.category.name.clone()),
        tags: post.tags.iter().map(|t| t.name.clone()).collect(),
    };

    SeoMetadataBuilder::new()
        .title(&post.title)
        .description(
            post.excerpt
                .as_deref()
                .unwrap_or("Read this post on Hmziq.rs Blog"),
        )
        .canonical(&format!("/posts/{}", post.slug))
        .image_struct(image)
        .article(article)
        .build()
}
#[component]
pub fn PostViewScreen(slug: String) -> Element {
    let nav = use_navigator();

    #[cfg(not(feature = "demo-static-content"))]
    let post_result = use_server_future(move || {
        let slug = slug.clone();
        async move { fetch_post_by_slug(slug).await }
    })?;
    #[cfg(not(feature = "demo-static-content"))]
    let post_state = post_result();

    #[cfg(feature = "demo-static-content")]
    let post_state = Some(Ok::<_, ServerFnError>(
        demo_content::content().post_by_slug(&slug),
    ));

    // Only use likes and auth when engagement feature is enabled
    #[cfg(feature = "engagement")]
    let likes = use_likes();
    #[cfg(feature = "engagement")]
    let auth = use_auth();

    // Analytics: Track page view and time spent
    #[cfg(all(feature = "analytics", not(feature = "demo-static-content")))]
    {
        let route = match &post_state {
            Some(Ok(Some(post))) => format!("/posts/{}", post.slug),
            _ => "/posts/unknown".to_string(),
        };
        use_page_timer(&route);
        use_scroll_depth(&route);

        let tracking_post = match &post_state {
            Some(Ok(Some(post))) => Some((
                post.id.to_string(),
                post.title.clone(),
                post.category.name.clone(),
            )),
            _ => None,
        };

        // Track post view when post is loaded
        use_effect(move || {
            if let Some((post_id, post_title, category_name)) = tracking_post.as_ref() {
                tracker::track_post_view(post_id, post_title, Some(category_name));
            }
        });
    }

    #[cfg(all(feature = "analytics", feature = "demo-static-content"))]
    {
        let route = format!("/posts/{}", slug);
        use_page_timer(&route);
        use_scroll_depth(&route);
    }

    match post_state {
        Some(Ok(Some(post))) => {
            // Generate SEO metadata
            let seo_metadata = generate_post_seo(&post);

            // Fetch like status when engagement feature is enabled
            #[cfg(feature = "engagement")]
            {
                use_effect(move || {
                    let is_logged_in = auth.user.read().is_some();
                    let status_is_init = likes
                        .status
                        .read()
                        .get(&post.id)
                        .map(|frame| frame.is_init())
                        .unwrap_or(true);

                    if is_logged_in && status_is_init {
                        let likes_state = likes;
                        let post_id = post.id;
                        spawn(async move {
                            likes_state.fetch_status(post_id).await;
                        });
                    }
                });
            }

            // Get like status from store (only when engagement feature is enabled)
            #[cfg(feature = "engagement")]
            let like_status = {
                let status_map = likes.status.read();
                status_map
                    .get(&post.id)
                    .and_then(|frame| frame.data.clone())
            };

            // Check if like action is loading (only when engagement feature is enabled)
            #[cfg(feature = "engagement")]
            let is_like_loading = {
                let action_map = likes.action.read();
                action_map
                    .get(&post.id)
                    .map(|frame| frame.is_loading())
                    .unwrap_or(false)
            };

            // Conditionally compile comments section (requires numeric post id)
            let comments_section: Option<Element> = {
                #[cfg(feature = "comments")]
                {
                    Some(rsx! {
                        div { id: "comments-section", class: "mb-12",
                            CommentsSection { post_id: post.id }
                        }
                    })
                }
                #[cfg(not(feature = "comments"))]
                {
                    None
                }
            };

            let published_date = post
                .published_at
                .as_ref()
                .map(|dt| format_date(dt))
                .unwrap_or_else(|| format_date(&post.created_at));

            let reading_time = estimate_reading_time(&post.content);

            // Get likes data - prefer from store if available, fallback to post data (only when engagement feature is enabled)
            #[cfg(feature = "engagement")]
            let (is_liked, likes_count) = match like_status {
                Some(status) => (status.is_liked, status.likes_count),
                None => (false, post.likes_count),
            };

            // Handle like toggle (only when engagement feature is enabled)
            #[cfg(feature = "engagement")]
            let handle_like = {
                let post_id = post.id;
                move |_| {
                    let is_logged_in = auth.user.read().is_some();
                    if !is_logged_in {
                        dioxus::logger::tracing::info!("User must be logged in to like posts");
                        return;
                    }

                    let likes_state = likes;
                    spawn(async move {
                        likes_state.toggle(post_id).await;
                    });
                }
            };

            // Handle scroll to comments (only when engagement feature is enabled)
            #[cfg(feature = "engagement")]
            let handle_scroll_to_comments = move |_| {
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                        if let Some(element) = document.get_element_by_id("comments-section") {
                            element.scroll_into_view();
                        }
                    }
                }
            };

            #[cfg(feature = "engagement")]
            let engagement_bar: Option<Element> = Some(rsx! {
                EngagementBar {
                    view_count: post.view_count,
                    likes_count,
                    comment_count: post.comment_count,
                    is_liked,
                    is_like_loading,
                    on_like: handle_like,
                    on_scroll_to_comments: handle_scroll_to_comments,
                    post_id: post.id.to_string(),
                    post_title: post.title.clone(),
                }
            });

            #[cfg(not(feature = "engagement"))]
            let engagement_bar: Option<Element> = None;

            let post_url = crate::seo::canonical_url(&format!("/posts/{}", post.slug));

            rsx! {
                // Inject SEO tags
                SeoHead { metadata: seo_metadata }

                // Inject breadcrumb structured data
                StructuredData {
                    json_ld: breadcrumb_schema(vec![
                        ("Home", "/"),
                        (&post.title, &format!("/posts/{}", post.slug))
                    ])
                }

                // Inject article structured data
                StructuredData { json_ld: article_schema(&post) }

                div { class: "min-h-screen",
                    BannerPlaceholder {}

                    div { class: "container mx-auto px-4 py-6 max-w-6xl",
                        // Post header
                        header { class: "mb-8",
                            // Category and tags
                            div { class: "flex flex-wrap gap-2 mb-4",
                                Link {
                                    to: crate::router::Route::CategoryDetailScreen { slug: post.category.slug.clone() },
                                    class: "category-pill hover:opacity-90 transition-opacity",
                                    "{post.category.name}"
                                }
                                for tag in post.tags.clone() {
                                    Link {
                                        to: crate::router::Route::TagDetailScreen { slug: tag.slug.clone() },
                                        class: "tag-chip",
                                        "{tag.name}"
                                    }
                                }
                            }

                            // Title
                            h1 { class: "text-3xl md:text-4xl font-bold leading-tight mb-4",
                                "{post.title}"
                            }

                            // Meta info
                            div { class: "flex flex-wrap items-center gap-4 text-sm text-muted-foreground",
                                div { class: "flex items-center gap-2",
                                    Icon { icon: LdCalendar, class: "w-4 h-4" }
                                    span { "{published_date}" }
                                }
                                div { class: "flex items-center gap-2",
                                    Icon { icon: LdClock, class: "w-4 h-4" }
                                    span { "{reading_time} min read" }
                                }
                                div { class: "flex items-center gap-2",
                                    span { "By {post.author.name}" }
                                }
                            }
                        }

                        // Featured image
                        if let Some(img) = &post.featured_image {
                            div { class: "mb-8",
                                img {
                                    src: "{img.file_url}",
                                    alt: "{post.title}",
                                    class: "w-full rounded-2xl border border-border/40 shadow-sm",
                                }
                            }
                        }

                        // Content
                        article { class: "prose prose-neutral dark:prose-invert max-w-none",
                            {render_editorjs_content(&post.content)}
                        }

                        {engagement_bar}

                        // Comments section
                        {comments_section}

                        // Action bar
                        ActionBar {
                            post_id: post.id.to_string(),
                            title: post.title.clone(),
                            url: post_url,
                        }

                        div { class: "flex justify-center pt-8",
                            Button {
                                variant: ButtonVariant::Ghost,
                                class: "h-10 px-4 rounded-lg hover:bg-muted/60",
                                onclick: move |_| { nav.push(crate::router::Route::HomeScreen {}); },
                                Icon { icon: LdArrowLeft, class: "w-4 h-4" }
                                "Back to all posts"
                            }
                        }
                    }
                }
            }
        }
        Some(Ok(None)) => {
            // Post not found
            rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    div { class: "text-center max-w-md",
                        h1 { class: "text-2xl font-bold mb-2", "Post not found" }
                        p { class: "text-muted-foreground mb-4", "The post you're looking for doesn't exist." }
                        button {
                            class: "text-primary hover:underline",
                            onclick: move |_| { nav.push(crate::router::Route::HomeScreen {}); },
                            "Back to home"
                        }
                    }
                }
            }
        }
        Some(Err(e)) => {
            // Error state
            rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    div { class: "text-center max-w-md",
                        h1 { class: "text-2xl font-bold mb-2", "Error loading post" }
                        p { class: "text-muted-foreground mb-4", "{e}" }
                        button {
                            class: "text-primary hover:underline",
                            onclick: move |_| { nav.push(crate::router::Route::HomeScreen {}); },
                            "Back to home"
                        }
                    }
                }
            }
        }
        None => {
            // Loading state
            rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    div { class: "animate-pulse text-muted-foreground", "Loading..." }
                }
            }
        }
    }
}
