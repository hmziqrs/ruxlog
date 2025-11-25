use dioxus::prelude::*;
use ruxlog_shared::store::{use_auth, use_likes, use_post};
use crate::utils::editorjs::render_editorjs_content;
use crate::components::{CommentsSection, EngagementBar, estimate_reading_time, format_date};
use hmziq_dioxus_free_icons::icons::ld_icons::{LdCalendar, LdClock, LdArrowLeft};
use hmziq_dioxus_free_icons::Icon;

#[component]
pub fn PostViewScreen(id: i32) -> Element {
    let posts = use_post();
    let likes = use_likes();
    let auth = use_auth();
    let nav = use_navigator();

    // Get post by id
    let post = use_memo(move || {
        let posts_read = posts.list.read();
        if let Some(list) = &(*posts_read).data {
            list.data.iter().find(|p| p.id == id).cloned()
        } else {
            None
        }
    });

    // Fetch posts if not loaded
    use_effect(move || {
        if post().is_none() {
            let posts_state = posts;
            spawn(async move {
                posts_state.list().await;
            });
        }
    });

    // Fetch like status when post is loaded and user is logged in
    use_effect(move || {
        let is_logged_in = auth.user.read().is_some();
        if is_logged_in && post().is_some() {
            let likes_state = likes;
            spawn(async move {
                likes_state.fetch_status(id).await;
            });
        }
    });

    // Get like status from store
    let like_status = use_memo(move || {
        let status_map = likes.status.read();
        status_map.get(&id).and_then(|frame| frame.data.clone())
    });

    // Check if like action is loading
    let is_like_loading = use_memo(move || {
        let action_map = likes.action.read();
        action_map
            .get(&id)
            .map(|frame| frame.is_loading())
            .unwrap_or(false)
    });

    // Handle like toggle
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

    // Handle share
    let handle_share = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let url = window.location().href().unwrap_or_default();
                let _ = window.navigator().clipboard().write_text(&url);
            }
        }
    };

    // Handle scroll to comments
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

    if let Some(post) = post() {
        let published_date = post
            .published_at
            .as_ref()
            .map(|dt| format_date(dt))
            .unwrap_or_else(|| format_date(&post.created_at));

        let reading_time = estimate_reading_time(&post.content);
        let post_id = post.id;

        // Get likes data - prefer from store if available, fallback to post data
        let (is_liked, likes_count) = match like_status() {
            Some(status) => (status.is_liked, status.likes_count),
            None => (false, post.likes_count),
        };

        rsx! {
            div { class: "min-h-screen bg-background text-foreground",
                // Article header
                header { class: "container mx-auto px-4 max-w-4xl pt-12 pb-8",
                    // Category & Tags
                    div { class: "flex flex-wrap items-center gap-2 mb-6 text-sm",
                        // Category
                        span { class: "text-primary font-medium", "{post.category.name}" }

                        // Tags (if any)
                        if !post.tags.is_empty() {
                            span { class: "text-muted-foreground", "·" }
                            for (i , tag) in post.tags.iter().take(2).enumerate() {
                                span { class: "text-muted-foreground", "{tag.name}" }
                                if i < post.tags.len().min(2) - 1 {
                                    span { class: "text-muted-foreground", "·" }
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
                        p { class: "text-lg text-muted-foreground leading-relaxed mb-8",
                            "{excerpt}"
                        }
                    }

                    // Author & Meta
                    div { class: "flex flex-wrap items-center gap-4 text-sm text-muted-foreground",
                        // Author
                        div { class: "flex items-center gap-2",
                            div { class: "w-8 h-8 rounded-full bg-muted flex items-center justify-center text-sm font-medium text-foreground",
                                "{post.author.name.chars().next().unwrap_or('U').to_uppercase()}"
                            }
                            span { class: "font-medium text-foreground", "{post.author.name}" }
                        }

                        span { "·" }

                        // Date
                        div { class: "flex items-center gap-1",
                            Icon { icon: LdCalendar, class: "w-4 h-4" }
                            span { "{published_date}" }
                        }

                        span { "·" }

                        // Reading time
                        div { class: "flex items-center gap-1",
                            Icon { icon: LdClock, class: "w-4 h-4" }
                            span { "{reading_time} min read" }
                        }
                    }
                }

                // Featured image
                if let Some(image) = &post.featured_image {
                    div { class: "container mx-auto px-4 max-w-4xl mb-10",
                        img {
                            src: "{image.file_url}",
                            alt: "{post.title}",
                            class: "w-full rounded-lg",
                        }
                    }
                }

                // Main content
                article { class: "container mx-auto px-4 max-w-4xl",
                    // Prose content
                    div { class: "prose prose-lg prose-neutral dark:prose-invert max-w-none
                        prose-headings:font-bold prose-headings:tracking-tight
                        prose-h2:text-2xl prose-h2:mt-10 prose-h2:mb-4
                        prose-h3:text-xl prose-h3:mt-8 prose-h3:mb-3
                        prose-p:leading-relaxed prose-p:text-muted-foreground
                        prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                        prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm
                        prose-pre:bg-muted prose-pre:border prose-pre:border-border
                        prose-img:rounded-lg
                        prose-blockquote:border-l-primary prose-blockquote:pl-4 prose-blockquote:italic
                        prose-li:text-muted-foreground
                        mb-12",
                        {render_editorjs_content(&post.content)}
                    }

                    // Engagement bar
                    div { class: "py-6 border-y border-border mb-12",
                        EngagementBar {
                            view_count: post.view_count,
                            likes_count,
                            comment_count: post.comment_count,
                            is_liked,
                            is_like_loading: is_like_loading(),
                            on_like: handle_like,
                            on_share: handle_share,
                            on_scroll_to_comments: handle_scroll_to_comments,
                        }
                    }

                    // Author section
                    div { class: "mb-12",
                        div { class: "flex items-start gap-4",
                            div { class: "w-14 h-14 rounded-full bg-muted flex items-center justify-center text-xl font-semibold text-foreground flex-shrink-0",
                                "{post.author.name.chars().next().unwrap_or('U').to_uppercase()}"
                            }
                            div {
                                div { class: "text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1",
                                    "Written by"
                                }
                                div { class: "text-lg font-semibold text-foreground mb-2",
                                    "{post.author.name}"
                                }
                                p { class: "text-sm text-muted-foreground leading-relaxed",
                                    "Thanks for reading! If you found this article helpful, consider sharing it with others."
                                }
                            }
                        }
                    }

                    // Comments section
                    div { id: "comments-section", class: "mb-12",
                        CommentsSection { post_id }
                    }
                }

                // Back button
                div { class: "container mx-auto px-4 max-w-4xl pb-16 pt-4",
                    button {
                        class: "flex items-center gap-2 mx-auto text-muted-foreground hover:text-foreground transition-colors group",
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
            div { class: "min-h-screen bg-background text-foreground",
                div { class: "container mx-auto px-4 max-w-4xl pt-12 pb-12",
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
