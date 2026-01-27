use crate::components::{estimate_reading_time, format_date, ActionBar};
use crate::utils::editorjs::render_editorjs_content;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdArrowLeft, LdCalendar, LdClock};
use hmziq_dioxus_free_icons::Icon;
use ruxlog_shared::store::use_post;

#[cfg(debug_assertions)]
use crate::hooks::use_unique_id;
#[cfg(debug_assertions)]
use std::{cell::Cell, rc::Rc};

#[cfg(feature = "engagement")]
use crate::components::EngagementBar;
#[cfg(feature = "engagement")]
use ruxlog_shared::store::{use_auth, use_likes};

#[cfg(feature = "comments")]
use crate::components::CommentsSection;

#[cfg(feature = "analytics")]
use crate::analytics::{tracker, use_page_timer, use_scroll_depth};

#[component]
pub fn PostViewScreen(id: i32) -> Element {
    let posts = use_post();
    let nav = use_navigator();

    #[cfg(debug_assertions)]
    let instance_id = use_unique_id();
    #[cfg(debug_assertions)]
    let instance_id_s = instance_id.read().clone();
    #[cfg(debug_assertions)]
    let render_counter = use_hook(|| Rc::new(Cell::new(0u32)));
    #[cfg(debug_assertions)]
    let effect_counter = use_hook(|| Rc::new(Cell::new(0u32)));

    // Only use likes and auth when engagement feature is enabled
    #[cfg(feature = "engagement")]
    let likes = use_likes();
    #[cfg(feature = "engagement")]
    let auth = use_auth();
    #[cfg(all(feature = "engagement", debug_assertions))]
    let likes_effect_counter = use_hook(|| Rc::new(Cell::new(0u32)));

    // Get post by id
    let post = use_memo(move || {
        let posts_read = posts.list.read();
        if let Some(list) = &(*posts_read).data {
            list.data.iter().find(|p| p.id == id).cloned()
        } else {
            None
        }
    });

    let post_data = post();

    #[cfg(debug_assertions)]
    {
        let n = render_counter.get().wrapping_add(1);
        render_counter.set(n);

        if n <= 20 || n % 50 == 0 {
            let list_frame = posts.list.read();
            dioxus::logger::tracing::info!(
                target: "ruxlog::ui",
                screen = "PostViewScreen",
                instance_id = %instance_id_s,
                post_id = id,
                render_count = n,
                post_in_list = post_data.is_some(),
                list_status = ?list_frame.status,
                list_is_loading = list_frame.is_loading(),
                list_is_failed = list_frame.is_failed(),
                list_error = ?list_frame.error_message(),
                list_has_data = list_frame.data.is_some(),
            );
        }
    }

    // Conditionally compile comments section
    let comments_section: Option<Element> = {
        #[cfg(feature = "comments")]
        {
            Some(rsx! {
                div { id: "comments-section", class: "mb-12",
                    CommentsSection { post_id: id }
                }
            })
        }
        #[cfg(not(feature = "comments"))]
        {
            None
        }
    };

    // Fetch posts if not loaded
    use_effect(move || {
        let list_frame = posts.list.read();
        let post_is_some = post().is_some();

        #[cfg(debug_assertions)]
        {
            let n = effect_counter.get().wrapping_add(1);
            effect_counter.set(n);

            if n <= 20 || n % 50 == 0 {
                dioxus::logger::tracing::info!(
                    target: "ruxlog::ui",
                    screen = "PostViewScreen",
                    instance_id = %instance_id_s,
                    post_id = id,
                    effect = "maybe_fetch_posts_list",
                    effect_run = n,
                    post_in_list = post_is_some,
                    list_status = ?list_frame.status,
                    list_is_loading = list_frame.is_loading(),
                    list_is_failed = list_frame.is_failed(),
                    list_error = ?list_frame.error_message(),
                );
            }
        }

        // Only kick off the list request once (Init -> Loading). This prevents
        // rapid re-renders from spawning multiple concurrent fetches.
        if !post_is_some && list_frame.is_init() {
            let posts_state = posts;
            spawn(async move {
                posts_state.list().await;
            });
        }
    });

    // Fetch like status when post is loaded and user is logged in (only when engagement feature is enabled)
    #[cfg(feature = "engagement")]
    use_effect(move || {
        let is_logged_in = auth.user.read().is_some();
        let post_is_some = post().is_some();
        let status_is_init = likes
            .status
            .read()
            .get(&id)
            .map(|frame| frame.is_init())
            .unwrap_or(true);

        #[cfg(debug_assertions)]
        {
            let n = likes_effect_counter.get().wrapping_add(1);
            likes_effect_counter.set(n);

            if n <= 20 || n % 50 == 0 {
                dioxus::logger::tracing::info!(
                    target: "ruxlog::ui",
                    screen = "PostViewScreen",
                    instance_id = %instance_id_s,
                    post_id = id,
                    effect = "maybe_fetch_like_status",
                    effect_run = n,
                    is_logged_in,
                    post_in_list = post_is_some,
                    status_is_init,
                );
            }
        }

        // Avoid spawning a status fetch on every render (a status fetch writes to the likes store,
        // which triggers a rerender, which would otherwise spawn again).
        if is_logged_in && post_is_some && status_is_init {
            let likes_state = likes;
            spawn(async move {
                likes_state.fetch_status(id).await;
            });
        }
    });

    // Get like status from store (only when engagement feature is enabled)
    #[cfg(feature = "engagement")]
    let like_status = use_memo(move || {
        let status_map = likes.status.read();
        status_map.get(&id).and_then(|frame| frame.data.clone())
    });

    // Check if like action is loading (only when engagement feature is enabled)
    #[cfg(feature = "engagement")]
    let is_like_loading = use_memo(move || {
        let action_map = likes.action.read();
        action_map
            .get(&id)
            .map(|frame| frame.is_loading())
            .unwrap_or(false)
    });

    // Handle like toggle (only when engagement feature is enabled)
    #[cfg(feature = "engagement")]
    let handle_like = move |_| {
        let is_logged_in = auth.user.read().is_some();
        if !is_logged_in {
            dioxus::logger::tracing::info!("User must be logged in to like posts");
            return;
        }

        let likes_state = likes;
        spawn(async move {
            likes_state.toggle(id).await;
        });
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

    // Analytics: Track page view and time spent
    #[cfg(feature = "analytics")]
    {
        let route = format!("/post/{}", id);
        use_page_timer(&route);
        use_scroll_depth(&route);

        // Track post view when post is loaded
        use_effect(move || {
            if let Some(post_data) = post() {
                tracker::track_post_view(
                    &post_data.id.to_string(),
                    &post_data.title,
                    Some(&post_data.category.name),
                );
            }
        });
    }

    if let Some(post) = post_data {
        let published_date = post
            .published_at
            .as_ref()
            .map(|dt| format_date(dt))
            .unwrap_or_else(|| format_date(&post.created_at));

        let reading_time = estimate_reading_time(&post.content);

        // Get likes data - prefer from store if available, fallback to post data (only when engagement feature is enabled)
        #[cfg(feature = "engagement")]
        let (is_liked, likes_count) = match like_status() {
            Some(status) => (status.is_liked, status.likes_count),
            None => (false, post.likes_count),
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
                            is_like_loading: is_like_loading(),
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

        rsx! {
            div { class: "min-h-screen bg-background",
                // Article header
                header { class: "container mx-auto px-4 max-w-6xl pt-12 pb-8",
                    // Category & Tags
                    div { class: "flex flex-wrap items-center gap-3 mb-6",
                        // Category (with border)
                        button {
                            class: "category-pill",
                            onclick: move |_| {
                                #[cfg(feature = "analytics")]
                                tracker::track_category_click(&post.category.name, "post_view");

                                nav.push(crate::router::Route::CategoryDetailScreen {
                                    slug: post.category.slug.clone(),
                                });
                            },
                            "{post.category.name}"
                        }

                        // Tags (clickable)
                        if !post.tags.is_empty() {
                            for tag in post.tags.iter().take(2) {
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
                        "{post.title}"
                    }

                    // Excerpt
                    if let Some(excerpt) = &post.excerpt {
                        p { class: "text-lg leading-relaxed mb-8",
                            "{excerpt}"
                        }
                    }

                    // Author & Meta
                    div { class: "flex flex-wrap items-center gap-4 text-sm",
                        // Author
                        div { class: "flex items-center gap-2",
                            div { class: "w-8 h-8 rounded-full bg-muted flex items-center justify-center text-sm font-medium",
                                "{post.author.name.chars().next().unwrap_or('U').to_uppercase()}"
                            }
                            span { class: "font-medium", "{post.author.name}" }
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
                if let Some(image) = &post.featured_image {
                    div { class: "container mx-auto px-4 max-w-6xl mb-10",
                        img {
                            src: "{image.file_url}",
                            alt: "{post.title}",
                            class: "w-full rounded-lg",
                        }
                    }
                }

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
                        {render_editorjs_content(&post.content)}
                    }
                }

                // Engagement bar (conditionally compiled) - consistent container
                div { class: "container mx-auto px-4 max-w-6xl mb-12",
                    { engagement_element }
                }

                // Action bar with utilities - consistent container
                div { class: "container mx-auto px-4 max-w-6xl mb-12",
                    ActionBar {
                        post_id: post.id.to_string(),
                        title: post.title.clone(),
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
    } else {
        // Loading state
        rsx! {
            div { class: "min-h-screen bg-background",
                div { class: "container mx-auto px-4 max-w-6xl pt-12 pb-12",
                    // Skeleton tags
                    div { class: "flex gap-2 mb-6",
                        div { class: "h-5 w-16 bg-muted rounded animate-pulse" }
                        div { class: "h-5 w-20 bg-muted rounded animate-pulse" }
                    }

                    // Skeleton title
                    div { class: "space-y-3 mb-8",
                        div { class: "h-10 w-full bg-muted rounded animate-pulse" }
                        div { class: "h-10 w-3/4 bg-muted rounded animate-pulse" }
                    }

                    // Skeleton excerpt
                    div { class: "h-6 w-full bg-muted rounded animate-pulse mb-2" }
                    div { class: "h-6 w-2/3 bg-muted rounded animate-pulse mb-8" }

                    // Skeleton meta
                    div { class: "flex items-center gap-4 mb-12",
                        div { class: "w-8 h-8 rounded-full bg-muted animate-pulse" }
                        div { class: "h-4 w-24 bg-muted rounded animate-pulse" }
                        div { class: "h-4 w-20 bg-muted rounded animate-pulse" }
                    }

                    // Skeleton image
                    div { class: "aspect-video w-full bg-muted rounded-lg animate-pulse mb-10" }

                    // Skeleton content
                    div { class: "space-y-4",
                        div { class: "h-4 w-full bg-muted rounded animate-pulse" }
                        div { class: "h-4 w-full bg-muted rounded animate-pulse" }
                        div { class: "h-4 w-3/4 bg-muted rounded animate-pulse" }
                        div { class: "h-4 w-full bg-muted rounded animate-pulse" }
                        div { class: "h-4 w-5/6 bg-muted rounded animate-pulse" }
                    }
                }
            }
        }
    }
}
