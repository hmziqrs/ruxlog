use dioxus::prelude::*;
use ruxlog_shared::store::{use_auth, use_likes, use_post};
use crate::utils::editorjs::render_editorjs_content;
use crate::components::{CommentsSection, EngagementBar, estimate_reading_time};
use hmziq_dioxus_free_icons::icons::ld_icons::{LdCalendar, LdClock};
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
            // Could show a login prompt here
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
            .map(|dt| dt.format("%B %d, %Y").to_string())
            .unwrap_or_else(|| post.created_at.format("%B %d, %Y").to_string());

        let reading_time = estimate_reading_time(&post.content);
        let post_id = post.id;

        // Get likes data - prefer from store if available, fallback to post data
        let (is_liked, likes_count) = match like_status() {
            Some(status) => (status.is_liked, status.likes_count),
            None => (false, post.likes_count),
        };

        rsx! {
            div { class: "min-h-screen bg-background text-foreground",
                // Hero section with featured image
                if let Some(image) = &post.featured_image {
                    div { class: "relative w-full h-[60vh] overflow-hidden",
                        img {
                            src: "{image.file_url}",
                            alt: "{post.title}",
                            class: "w-full h-full object-cover"
                        }
                        div { class: "absolute inset-0 bg-gradient-to-t from-background via-background/60 to-transparent" }
                    }
                }

                // Content
                article { class: "container mx-auto px-4 -mt-32 relative z-10",
                    div { class: "max-w-4xl mx-auto",
                        // Title and metadata
                        header { class: "mb-8 bg-background/95 backdrop-blur-sm rounded-lg p-8 shadow-xl border border-border/50",
                            h1 { class: "text-4xl md:text-5xl font-bold mb-6 leading-tight", "{post.title}" }
                            
                            if let Some(excerpt) = &post.excerpt {
                                p { class: "text-xl text-muted-foreground mb-6 leading-relaxed", "{excerpt}" }
                            }

                            // Author and meta info
                            div { class: "flex flex-wrap items-center gap-4 text-sm text-muted-foreground",
                                // Author
                                div { class: "flex items-center gap-3",
                                    div { class: "w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-primary font-semibold",
                                        "{post.author.name.chars().next().unwrap_or('U').to_uppercase()}"
                                    }
                                    span { class: "font-medium text-foreground", "{post.author.name}" }
                                }
                                
                                span { class: "text-border", "·" }
                                
                                // Date
                                div { class: "flex items-center gap-1.5",
                                    Icon { icon: LdCalendar, class: "w-4 h-4" }
                                    span { "{published_date}" }
                                }
                                
                                span { class: "text-border", "·" }
                                
                                // Reading time
                                div { class: "flex items-center gap-1.5",
                                    Icon { icon: LdClock, class: "w-4 h-4" }
                                    span { "{reading_time} min read" }
                                }
                            }

                            // Tags
                            if !post.tags.is_empty() {
                                div { class: "flex flex-wrap gap-2 mt-6",
                                    for tag in &post.tags {
                                        span {
                                            class: "px-3 py-1 rounded-full text-xs font-medium bg-primary/10 text-primary",
                                            "{tag.name}"
                                        }
                                    }
                                }
                            }
                        }

                        // Post content
                        div { class: "bg-background/50 rounded-lg p-8 mb-8",
                            {render_editorjs_content(&post.content)}
                        }

                        // Engagement bar
                        EngagementBar {
                            view_count: post.view_count,
                            likes_count: likes_count,
                            comment_count: post.comment_count,
                            is_liked: is_liked,
                            is_like_loading: is_like_loading(),
                            on_like: handle_like,
                            on_share: handle_share,
                            on_scroll_to_comments: handle_scroll_to_comments,
                        }

                        // Comments section
                        div { id: "comments-section",
                            CommentsSection { post_id: post_id }
                        }

                        // Navigation
                        div { class: "mt-12 pt-8 border-t border-border",
                            button {
                                onclick: move |_| { nav.push(crate::router::Route::HomeScreen {}); },
                                class: "text-primary hover:underline",
                                "← Back to all posts"
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "min-h-screen bg-background text-foreground flex items-center justify-center",
                div { class: "text-center",
                    div { class: "animate-pulse mb-4",
                        div { class: "w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin mx-auto" }
                    }
                    p { class: "text-muted-foreground", "Loading post..." }
                }
            }
        }
    }
}
