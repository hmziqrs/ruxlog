use dioxus::prelude::*;
use ruxlog_shared::store::use_post;
use crate::utils::editorjs::render_editorjs_content;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdCalendar, LdClock, LdEye, LdHeart, LdMessageCircle, LdShare2};
use hmziq_dioxus_free_icons::Icon;

#[component]
pub fn PostViewScreen(id: i32) -> Element {
    let posts = use_post();
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

    if let Some(post) = post() {
        let published_date = post
            .published_at
            .map(|dt| dt.format("%B %d, %Y").to_string())
            .unwrap_or_else(|| post.created_at.format("%B %d, %Y").to_string());

        let reading_time = estimate_reading_time(&post.content);

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
                                
                                span { class: "text-border", "•" }
                                
                                // Date
                                div { class: "flex items-center gap-1.5",
                                    Icon { icon: LdCalendar, class: "w-4 h-4" }
                                    span { "{published_date}" }
                                }
                                
                                span { class: "text-border", "•" }
                                
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
                                        a {
                                            href: "/tags/{tag.slug}",
                                            class: "px-3 py-1 rounded-full text-xs font-medium bg-primary/10 text-primary hover:bg-primary/20 transition-colors",
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
                        div { class: "flex items-center justify-between p-6 bg-muted/30 rounded-lg border border-border/50",
                            div { class: "flex items-center gap-6",
                                // Views
                                div { class: "flex items-center gap-2 text-muted-foreground",
                                    Icon { icon: LdEye, class: "w-5 h-5" }
                                    span { class: "font-medium", "{post.view_count}" }
                                }
                                
                                // Likes
                                div { class: "flex items-center gap-2 text-muted-foreground",
                                    Icon { icon: LdHeart, class: "w-5 h-5" }
                                    span { class: "font-medium", "{post.likes_count}" }
                                }
                                
                                // Comments
                                div { class: "flex items-center gap-2 text-muted-foreground",
                                    Icon { icon: LdMessageCircle, class: "w-5 h-5" }
                                    span { class: "font-medium", "{post.comment_count}" }
                                }
                            }

                            button {
                                class: "flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors",
                                Icon { icon: LdShare2, class: "w-4 h-4" }
                                "Share"
                            }
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

fn estimate_reading_time(content: &ruxlog_shared::store::PostContent) -> u32 {
    let word_count: usize = content.blocks.iter().map(|block| {
        match block {
            ruxlog_shared::store::EditorJsBlock::Paragraph { data, .. } => {
                data.text.split_whitespace().count()
            },
            ruxlog_shared::store::EditorJsBlock::Header { data, .. } => {
                data.text.split_whitespace().count()
            },
            ruxlog_shared::store::EditorJsBlock::List { data, .. } => {
                data.items.iter().map(|item| item.split_whitespace().count()).sum()
            },
            _ => 0,
        }
    }).sum();
    
    // Average reading speed is 200-250 words per minute
    ((word_count as f32 / 225.0).ceil() as u32).max(1)
}
