use crate::components::{estimate_reading_time, format_date, ActionBar, BannerPlaceholder};
use crate::seo::{
    article_schema, breadcrumb_schema, ArticleMetadata, SeoHead, SeoImage, SeoMetadataBuilder,
    StructuredData,
};
use crate::server_fns::fetch_post_by_slug;
use crate::utils::editorjs::render_editorjs_content;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdArrowLeft, LdCalendar, LdClock};
use hmziq_dioxus_free_icons::Icon;
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

    // SSR: Fetch post by slug on server
    let post_result = use_server_future(move || {
        let slug = slug.clone();
        async move { fetch_post_by_slug(slug).await }
    })?;

    // Only use likes and auth when engagement feature is enabled
    #[cfg(feature = "engagement")]
    let likes = use_likes();
    #[cfg(feature = "engagement")]
    let auth = use_auth();

    // Analytics: Track page view and time spent
    #[cfg(feature = "analytics")]
    {
        let route = match &post_result() {
            Some(Ok(Some(post))) => format!("/posts/{}", post.slug),
            _ => "/posts/unknown".to_string(),
        };
        use_page_timer(&route);
        use_scroll_depth(&route);

        // Track post view when post is loaded
        use_effect(move || {
            if let Some(Ok(Some(ref post_data))) = post_result() {
                tracker::track_post_view(
                    &post_data.id.to_string(),
                    &post_data.title,
                    Some(&post_data.category.name),
                );
            }
        });
    }

    match post_result() {
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

            // Conditionally compile engagement bar
            let engagement_element: Option<Element> = {
                #[cfg(feature = "engagement")]
                {
                    Some(rsx! {
                        div { class: "py-6 border-y border-border",
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
                        }
                    })
                }
                #[cfg(not(feature = "engagement"))]
                {
                    None
                }
            };

            // Clone values needed in closures
            let post_category_name = post.category.name.clone();
            let post_category_slug = post.category.slug.clone();
            let post_tags = post.tags.clone();
            let post_title = post.title.clone();
            let post_slug = post.slug.clone();
            let post_excerpt = post.excerpt.clone();
            let post_author = post.author.clone();
            let post_featured_image = post.featured_image.clone();
            let post_content = post.content.clone();
            let post_id = post.id;

            rsx! {
                // Inject SEO tags
                SeoHead { metadata: seo_metadata }

                // Inject structured data
                StructuredData { json_ld: article_schema(&post) }
                StructuredData {
                    json_ld: breadcrumb_schema(vec![
                        ("Home", "/"),
                        (&post_category_name, &format!("/categories/{}", post_category_slug)),
                        (&post_title, &format!("/posts/{}", post_slug))
                    ])
                }

                div { class: "min-h-screen",
                    // Article header
                    header { class: "container mx-auto px-4 max-w-6xl pt-12 pb-8",
                        // Category & Tags
                        div { class: "flex flex-wrap items-center gap-3 mb-6",
                            // Category (with border)
                            button {
                                class: "category-pill",
                                onclick: move |_| {
                                    #[cfg(feature = "analytics")]
                                    tracker::track_category_click(&post_category_name, "post_view");

                                    nav.push(crate::router::Route::CategoryDetailScreen {
                                        slug: post_category_slug.clone(),
                                    });
                                },
                                "{post.category.name}"
                            }

                            // Tags (clickable)
                            if !post_tags.is_empty() {
                                for tag in post_tags.iter().take(2) {
                                    {
                                        let tag_slug = tag.slug.clone();
                                        let tag_name = tag.name.clone();
                                        rsx! {
                                            button {
                                                class: "tag-badge",
                                                onclick: move |_| {
                                                    #[cfg(feature = "analytics")]
                                                    tracker::track_tag_click(&tag_name, "post_view");

                                                    nav.push(crate::router::Route::TagDetailScreen {
                                                        slug: tag_slug.clone(),
                                                    });
                                                },
                                                "{tag_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Title
                        h1 { class: "text-3xl sm:text-4xl md:text-5xl font-bold leading-tight tracking-tight mb-6",
                            "{post_title}"
                        }

                        // Excerpt
                        if let Some(excerpt) = &post_excerpt {
                            p { class: "text-lg leading-relaxed mb-8",
                                "{excerpt}"
                            }
                        }

                        // Author & Meta
                        div { class: "flex flex-wrap items-center gap-4 text-sm",
                            // Author
                            div { class: "flex items-center gap-2",
                                div { class: "w-8 h-8 rounded-full bg-muted flex items-center justify-center text-sm font-medium",
                                    "{post_author.name.chars().next().unwrap_or('U').to_uppercase()}"
                                }
                                span { class: "font-medium", "{post_author.name}" }
                            }

                            span { "·" }

                            // Date
                            div { class: "flex items-center gap-1",
                                Icon { icon: LdCalendar, class: "w-4 h-4" }
                                span { class: "meta-mono", "{published_date}" }
                            }

                            span { "·" }

                            // Reading time
                            div { class: "flex items-center gap-1",
                                Icon { icon: LdClock, class: "w-4 h-4" }
                                span { class: "meta-mono", "{reading_time} min read" }
                            }
                        }
                    }

                    // Featured image
                    if let Some(image) = &post_featured_image {
                        div { class: "container mx-auto px-4 max-w-6xl mb-10",
                            img {
                                src: "{image.file_url}",
                                alt: "{post_title}",
                                class: "w-full rounded-lg",
                            }
                        }
                    }

                    // Banner placeholder
                    BannerPlaceholder {}

                    // Main content
                    article { class: "container mx-auto px-4 max-w-6xl",
                        // Prose content
                        div { class: "prose prose-lg max-w-none
                            prose-headings:font-bold prose-headings:tracking-tight
                            prose-h2:text-2xl prose-h2:mt-10 prose-h2:mb-4
                            prose-h3:text-xl prose-h3:mt-8 prose-h3:mb-3
                            prose-p:leading-relaxed
                            prose-a:no-underline hover:prose-a:underline
                            prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm
                            prose-pre:bg-muted prose-pre:border prose-pre:border-border
                            prose-img:rounded-lg
                            prose-blockquote:border-l-primary prose-blockquote:pl-4 prose-blockquote:italic
                            mb-12",
                            {render_editorjs_content(&post_content)}
                        }
                    }

                    // Engagement bar (conditionally compiled) - consistent container
                    div { class: "container mx-auto px-4 max-w-6xl mb-12",
                        { engagement_element }
                    }

                    // Action bar with utilities - consistent container
                    div { class: "container mx-auto px-4 max-w-6xl mb-12",
                        ActionBar {
                            post_id: post_id.to_string(),
                            title: post_title.clone(),
                            url: {
                                #[cfg(target_arch = "wasm32")]
                                {
                                    web_sys::window()
                                        .and_then(|w| w.location().href().ok())
                                        .unwrap_or_default()
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    String::new()
                                }
                            },
                        }
                    }

                    // Comments section - consistent container
                    div { class: "container mx-auto px-4 max-w-6xl mb-12",
                        { comments_section }
                    }

                    // Back button
                    div { class: "container mx-auto px-4 max-w-6xl pb-16 pt-4",
                        button {
                            class: "flex items-center gap-2 mx-auto group",
                            onclick: move |_| {
                                nav.push(crate::router::Route::HomeScreen {
                                });
                            },
                            Icon {
                                icon: LdArrowLeft,
                                class: "w-4 h-4 transition-transform group-hover:-translate-x-1",
                            }
                            span { class: "text-sm", "Back to all posts" }
                        }
                    }
                }
            }
        }
        Some(Ok(None)) => {
            // Post not found
            rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    div { class: "text-center",
                        h1 { class: "text-2xl font-bold mb-4", "Post not found" }
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
                    div { class: "text-center",
                        h1 { class: "text-2xl font-bold mb-4", "Error loading post" }
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
