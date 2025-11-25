use dioxus::prelude::*;
use ruxlog_shared::store::{use_auth, use_likes, use_post};
use crate::utils::editorjs::render_editorjs_content;
use crate::components::{CommentsSection, EngagementBar, estimate_reading_time, format_date, get_gradient_for_tag};
use hmziq_dioxus_free_icons::icons::ld_icons::{LdCalendar, LdClock, LdArrowLeft, LdBookOpen};
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
        let first_tag = post.tags.first().map(|t| t.name.clone());
        let first_tag_str = first_tag.as_deref();
        let gradient = get_gradient_for_tag(first_tag_str);

        // Get likes data - prefer from store if available, fallback to post data
        let (is_liked, likes_count) = match like_status() {
            Some(status) => (status.is_liked, status.likes_count),
            None => (false, post.likes_count),
        };

        rsx! {
            div { class: "min-h-screen bg-background text-foreground",
                // Top navigation bar
                div { class: "sticky top-0 z-50 bg-background/80 backdrop-blur-md border-b border-border/50",
                    div { class: "container mx-auto px-4 max-w-6xl",
                        div { class: "flex items-center justify-between h-14",
                            button {
                                class: "flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors group",
                                onclick: move |_| { nav.push(crate::router::Route::HomeScreen {}); },
                                Icon { icon: LdArrowLeft, class: "w-4 h-4 transition-transform group-hover:-translate-x-1" }
                                span { class: "text-sm font-medium", "All posts" }
                            }

                            // Reading progress could go here
                            div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                                Icon { icon: LdBookOpen, class: "w-4 h-4" }
                                span { "{reading_time} min read" }
                            }
                        }
                    }
                }

                // Hero section
                div { class: "relative",
                    // Background image/gradient
                    div { class: "absolute inset-0 h-[50vh] overflow-hidden",
                        if let Some(image) = &post.featured_image {
                            img {
                                src: "{image.file_url}",
                                alt: "{post.title}",
                                class: "w-full h-full object-cover opacity-30"
                            }
                            div { class: "absolute inset-0 bg-gradient-to-b from-background/60 via-background/80 to-background" }
                        } else {
                            // Fallback gradient
                            div {
                                class: "w-full h-full bg-gradient-to-br {gradient}",
                                div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-primary/10 via-transparent to-transparent" }
                            }
                            div { class: "absolute inset-0 bg-gradient-to-b from-transparent via-background/50 to-background" }
                        }
                    }

                    // Hero content
                    div { class: "container mx-auto px-4 max-w-4xl relative pt-12 pb-8",
                        // Tags
                        if !post.tags.is_empty() {
                            div { class: "flex flex-wrap gap-2 mb-6",
                                for tag in post.tags.iter().take(3) {
                                    span { class: "px-3 py-1.5 rounded-full text-xs font-semibold bg-primary/10 text-primary border border-primary/20 backdrop-blur-sm",
                                        "{tag.name}"
                                    }
                                }
                            }
                        }

                        // Title
                        h1 { class: "text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-bold leading-[1.1] tracking-tight mb-6",
                            "{post.title}"
                        }

                        // Excerpt
                        if let Some(excerpt) = &post.excerpt {
                            p { class: "text-lg md:text-xl text-muted-foreground leading-relaxed mb-8 max-w-3xl",
                                "{excerpt}"
                            }
                        }

                        // Author & Meta
                        div { class: "flex flex-wrap items-center gap-6",
                            // Author
                            div { class: "flex items-center gap-3",
                                div { class: "w-12 h-12 rounded-full bg-gradient-to-br from-primary/30 to-primary/10 flex items-center justify-center text-lg font-bold text-primary ring-2 ring-background shadow-lg",
                                    "{post.author.name.chars().next().unwrap_or('U').to_uppercase()}"
                                }
                                div {
                                    div { class: "font-semibold text-foreground", "{post.author.name}" }
                                    div { class: "text-sm text-muted-foreground", "Author" }
                                }
                            }

                            div { class: "hidden sm:block w-px h-10 bg-border" }

                            // Date
                            div { class: "flex items-center gap-2 text-muted-foreground",
                                Icon { icon: LdCalendar, class: "w-4 h-4 text-primary/70" }
                                span { class: "text-sm", "{published_date}" }
                            }

                            div { class: "hidden sm:block w-px h-10 bg-border" }

                            // Reading time
                            div { class: "flex items-center gap-2 text-muted-foreground",
                                Icon { icon: LdClock, class: "w-4 h-4 text-primary/70" }
                                span { class: "text-sm", "{reading_time} min read" }
                            }
                        }
                    }
                }

                // Featured image card (if exists)
                if let Some(image) = &post.featured_image {
                    div { class: "container mx-auto px-4 max-w-4xl -mt-4 mb-8",
                        div { class: "rounded-2xl overflow-hidden border border-border/50 shadow-2xl shadow-black/10",
                            img {
                                src: "{image.file_url}",
                                alt: "{post.title}",
                                class: "w-full aspect-[21/9] object-cover"
                            }
                        }
                    }
                }

                // Main content
                article { class: "container mx-auto px-4 max-w-4xl",
                    // Article content with gradient border
                    div { class: "rounded-2xl bg-gradient-to-br from-border/50 via-border/30 to-transparent p-[1px] mb-8",
                        div { class: "rounded-2xl bg-card/50 backdrop-blur-sm p-6 sm:p-8 md:p-10",
                            // Prose content
                            div { class: "prose prose-lg prose-neutral dark:prose-invert max-w-none
                                prose-headings:font-bold prose-headings:tracking-tight
                                prose-h2:text-2xl prose-h2:mt-10 prose-h2:mb-4
                                prose-h3:text-xl prose-h3:mt-8 prose-h3:mb-3
                                prose-p:leading-relaxed prose-p:text-muted-foreground
                                prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                                prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm
                                prose-pre:bg-muted/50 prose-pre:border prose-pre:border-border/50
                                prose-img:rounded-xl prose-img:shadow-lg
                                prose-blockquote:border-l-primary prose-blockquote:bg-muted/30 prose-blockquote:rounded-r-lg prose-blockquote:py-1
                                prose-li:text-muted-foreground",
                                {render_editorjs_content(&post.content)}
                            }
                        }
                    }

                    // Engagement bar
                    div { class: "mb-8",
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
                    }

                    // Author bio card
                    div { class: "rounded-2xl bg-gradient-to-br from-primary/10 via-transparent to-transparent p-[1px] mb-8",
                        div { class: "rounded-2xl bg-card/50 backdrop-blur-sm p-6",
                            div { class: "flex items-start gap-4",
                                div { class: "w-16 h-16 rounded-full bg-gradient-to-br from-primary/30 to-primary/10 flex items-center justify-center text-2xl font-bold text-primary flex-shrink-0",
                                    "{post.author.name.chars().next().unwrap_or('U').to_uppercase()}"
                                }
                                div {
                                    div { class: "text-xs font-medium text-primary uppercase tracking-wider mb-1", "Written by" }
                                    div { class: "text-xl font-bold text-foreground mb-2", "{post.author.name}" }
                                    p { class: "text-sm text-muted-foreground leading-relaxed",
                                        "Thanks for reading! If you found this article helpful, consider sharing it with others."
                                    }
                                }
                            }
                        }
                    }

                    // Comments section
                    div { id: "comments-section", class: "mb-12",
                        CommentsSection { post_id: post_id }
                    }
                }

                // Footer navigation
                div { class: "border-t border-border/50 bg-muted/20",
                    div { class: "container mx-auto px-4 max-w-4xl py-8",
                        button {
                            class: "flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors group",
                            onclick: move |_| { nav.push(crate::router::Route::HomeScreen {}); },
                            Icon { icon: LdArrowLeft, class: "w-4 h-4 transition-transform group-hover:-translate-x-1" }
                            span { class: "font-medium", "Back to all posts" }
                        }
                    }
                }
            }
        }
    } else {
        // Loading state
        rsx! {
            div { class: "min-h-screen bg-background text-foreground",
                // Skeleton header
                div { class: "sticky top-0 z-50 bg-background/80 backdrop-blur-md border-b border-border/50",
                    div { class: "container mx-auto px-4 max-w-6xl",
                        div { class: "flex items-center justify-between h-14",
                            div { class: "h-4 w-24 bg-muted/50 rounded animate-pulse" }
                            div { class: "h-4 w-20 bg-muted/50 rounded animate-pulse" }
                        }
                    }
                }

                div { class: "container mx-auto px-4 max-w-4xl py-12",
                    // Skeleton tags
                    div { class: "flex gap-2 mb-6",
                        div { class: "h-7 w-20 bg-muted/50 rounded-full animate-pulse" }
                        div { class: "h-7 w-24 bg-muted/50 rounded-full animate-pulse" }
                    }

                    // Skeleton title
                    div { class: "space-y-3 mb-8",
                        div { class: "h-12 w-full bg-muted/50 rounded animate-pulse" }
                        div { class: "h-12 w-3/4 bg-muted/50 rounded animate-pulse" }
                    }

                    // Skeleton excerpt
                    div { class: "h-6 w-full bg-muted/40 rounded animate-pulse mb-2" }
                    div { class: "h-6 w-2/3 bg-muted/40 rounded animate-pulse mb-8" }

                    // Skeleton meta
                    div { class: "flex items-center gap-4 mb-12",
                        div { class: "w-12 h-12 rounded-full bg-muted/50 animate-pulse" }
                        div { class: "space-y-2",
                            div { class: "h-4 w-32 bg-muted/50 rounded animate-pulse" }
                            div { class: "h-3 w-20 bg-muted/40 rounded animate-pulse" }
                        }
                    }

                    // Skeleton image
                    div { class: "aspect-[21/9] w-full bg-muted/30 rounded-2xl animate-pulse mb-8" }

                    // Skeleton content
                    div { class: "space-y-4",
                        div { class: "h-4 w-full bg-muted/40 rounded animate-pulse" }
                        div { class: "h-4 w-full bg-muted/40 rounded animate-pulse" }
                        div { class: "h-4 w-3/4 bg-muted/40 rounded animate-pulse" }
                        div { class: "h-4 w-full bg-muted/40 rounded animate-pulse" }
                        div { class: "h-4 w-5/6 bg-muted/40 rounded animate-pulse" }
                    }
                }
            }
        }
    }
}
