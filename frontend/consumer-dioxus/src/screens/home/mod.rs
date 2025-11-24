use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::use_post;
use ruxlog_shared::store::posts::PostContent;
use crate::router::Route;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdCalendar, LdClock, LdArrowRight, LdEye};
use hmziq_dioxus_free_icons::Icon;
use chrono::{DateTime, Utc};

/// Estimate reading time based on content blocks (avg 200 words per minute)
fn estimate_reading_time(content: &PostContent) -> u32 {
    // Extract text from blocks and count words
    let mut word_count = 0;
    for block in &content.blocks {
        let text = match block {
            ruxlog_shared::store::posts::EditorJsBlock::Header { data, .. } => &data.text,
            ruxlog_shared::store::posts::EditorJsBlock::Paragraph { data, .. } => &data.text,
            ruxlog_shared::store::posts::EditorJsBlock::List { data, .. } => {
                word_count += data.items.iter().map(|s| s.split_whitespace().count()).sum::<usize>();
                continue;
            }
            _ => continue,
        };
        word_count += text.split_whitespace().count();
    }
    let minutes = (word_count as f64 / 200.0).ceil() as u32;
    minutes.max(1)
}

/// Format DateTime to a readable string
fn format_date(date: &DateTime<Utc>) -> String {
    date.format("%b %d, %Y").to_string()
}

#[component]
pub fn HomeScreen() -> Element {
    let posts_store = use_post();

    // Fetch posts on mount
    use_effect(move || {
        let posts = posts_store;
        spawn(async move {
            posts.list().await;
        });
    });

    let posts_frame = posts_store.list.read();

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            // Hero Section
            div { class: "border-b border-border/60",
                div { class: "container mx-auto px-4 py-16 md:py-24",
                    div { class: "max-w-3xl",
                        h1 { class: "text-4xl md:text-5xl lg:text-6xl font-bold tracking-tight mb-6",
                            "Engineering "
                            span { class: "text-primary", "Insights" }
                        }
                        p { class: "text-lg md:text-xl text-muted-foreground leading-relaxed",
                            "Deep dives into software architecture, system design, and the craft of building scalable applications. Written for engineers who care about the details."
                        }
                    }
                }
            }

            // Posts Section
            div { class: "container mx-auto px-4 py-12 md:py-16",
                if (*posts_frame).is_loading() {
                    // Loading skeleton
                    div { class: "space-y-6",
                        for _ in 0..3 {
                            div { class: "animate-pulse",
                                div { class: "rounded-lg border border-border/40 bg-card/30 p-6 md:p-8",
                                    div { class: "flex flex-col md:flex-row gap-6",
                                        div { class: "flex-1 space-y-4",
                                            div { class: "h-4 bg-muted/50 rounded w-24" }
                                            div { class: "h-8 bg-muted/50 rounded w-3/4" }
                                            div { class: "h-4 bg-muted/50 rounded w-full" }
                                            div { class: "h-4 bg-muted/50 rounded w-2/3" }
                                        }
                                        div { class: "w-full md:w-64 h-40 bg-muted/50 rounded-lg" }
                                    }
                                }
                            }
                        }
                    }
                } else if (*posts_frame).is_failed() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "max-w-md w-full",
                            ErrorDetails {
                                error: (*posts_frame).error.clone(),
                                variant: ErrorDetailsVariant::Collapsed,
                            }
                        }
                    }
                } else if let Some(data) = &(*posts_frame).data {
                    if data.data.is_empty() {
                        // Empty state
                        div { class: "flex flex-col items-center justify-center py-20 text-center",
                            div { class: "w-16 h-16 rounded-full bg-muted/50 flex items-center justify-center mb-6",
                                Icon { icon: LdCalendar, class: "w-8 h-8 text-muted-foreground" }
                            }
                            h3 { class: "text-xl font-semibold mb-2", "No posts yet" }
                            p { class: "text-muted-foreground max-w-sm",
                                "Check back soon for new articles on software engineering, architecture, and best practices."
                            }
                        }
                    } else {
                        // Featured post (first post)
                        if let Some(featured) = data.data.first() {
                            div { class: "mb-12",
                                Link {
                                    to: Route::PostViewScreen { id: featured.id },
                                    class: "group block",
                                    div { class: "rounded-xl border border-border/60 bg-card/50 backdrop-blur-sm overflow-hidden transition-all duration-300 hover:border-border hover:bg-card/80 hover:shadow-lg",
                                        div { class: "flex flex-col lg:flex-row",
                                            // Featured image
                                            if let Some(img) = &featured.featured_image {
                                                div { class: "lg:w-1/2 aspect-video lg:aspect-auto",
                                                    img {
                                                        src: "{img.file_url}",
                                                        alt: "{featured.title}",
                                                        class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500",
                                                    }
                                                }
                                            }

                                            // Content
                                            div { class: "flex-1 p-6 md:p-8 lg:p-10 flex flex-col justify-center",
                                                // Label
                                                div { class: "flex items-center gap-3 mb-4",
                                                    span { class: "px-3 py-1 rounded-full text-xs font-medium bg-primary/10 text-primary border border-primary/20",
                                                        "Featured"
                                                    }
                                                    if !featured.tags.is_empty() {
                                                        span { class: "text-muted-foreground text-sm",
                                                            "{featured.tags.first().map(|t| t.name.as_str()).unwrap_or(\"\")}"
                                                        }
                                                    }
                                                }

                                                h2 { class: "text-2xl md:text-3xl lg:text-4xl font-bold mb-4 group-hover:text-primary transition-colors leading-tight",
                                                    "{featured.title}"
                                                }

                                                if let Some(excerpt) = &featured.excerpt {
                                                    p { class: "text-muted-foreground text-base md:text-lg leading-relaxed mb-6 line-clamp-3",
                                                        "{excerpt}"
                                                    }
                                                }

                                                // Meta info
                                                div { class: "flex flex-wrap items-center gap-4 text-sm text-muted-foreground",
                                                    // Author
                                                    div { class: "flex items-center gap-2",
                                                        div { class: "w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center text-xs font-semibold text-primary",
                                                            "{featured.author.name.chars().next().unwrap_or('A').to_uppercase()}"
                                                        }
                                                        span { "{featured.author.name}" }
                                                    }

                                                    span { class: "text-border", "•" }

                                                    // Date
                                                    if let Some(published) = &featured.published_at {
                                                        div { class: "flex items-center gap-1.5",
                                                            Icon { icon: LdCalendar, class: "w-4 h-4" }
                                                            span { "{format_date(published)}" }
                                                        }
                                                    }

                                                    span { class: "text-border", "•" }

                                                    // Reading time
                                                    div { class: "flex items-center gap-1.5",
                                                        Icon { icon: LdClock, class: "w-4 h-4" }
                                                        span { "{estimate_reading_time(&featured.content)} min read" }
                                                    }
                                                }

                                                // Read more indicator
                                                div { class: "mt-6 flex items-center gap-2 text-primary font-medium group-hover:gap-3 transition-all",
                                                    span { "Read article" }
                                                    Icon { icon: LdArrowRight, class: "w-4 h-4" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Section header
                        if data.data.len() > 1 {
                            div { class: "flex items-center justify-between mb-8",
                                h2 { class: "text-2xl font-bold", "Latest Articles" }
                                div { class: "h-px flex-1 bg-border/60 ml-6" }
                            }
                        }

                        // Posts list (excluding featured)
                        div { class: "space-y-4",
                            for post in data.data.iter().skip(1) {
                                Link {
                                    to: Route::PostViewScreen { id: post.id },
                                    class: "group block",
                                    article { class: "rounded-lg border border-border/40 bg-transparent hover:bg-card/50 hover:border-border/80 transition-all duration-300 p-6",
                                        div { class: "flex flex-col md:flex-row gap-6",
                                            // Text content
                                            div { class: "flex-1 min-w-0",
                                                // Tags
                                                if !post.tags.is_empty() {
                                                    div { class: "flex flex-wrap gap-2 mb-3",
                                                        for tag in post.tags.iter().take(2) {
                                                            span { class: "px-2 py-0.5 rounded text-xs font-medium bg-muted/50 text-muted-foreground",
                                                                "{tag.name}"
                                                            }
                                                        }
                                                    }
                                                }

                                                h3 { class: "text-xl font-semibold mb-2 group-hover:text-primary transition-colors line-clamp-2",
                                                    "{post.title}"
                                                }

                                                if let Some(excerpt) = &post.excerpt {
                                                    p { class: "text-muted-foreground text-sm leading-relaxed mb-4 line-clamp-2",
                                                        "{excerpt}"
                                                    }
                                                }

                                                // Meta
                                                div { class: "flex flex-wrap items-center gap-3 text-xs text-muted-foreground",
                                                    span { class: "font-medium", "{post.author.name}" }

                                                    if let Some(published) = &post.published_at {
                                                        span { class: "text-border", "•" }
                                                        span { "{format_date(published)}" }
                                                    }

                                                    span { class: "text-border", "•" }

                                                    div { class: "flex items-center gap-1",
                                                        Icon { icon: LdClock, class: "w-3 h-3" }
                                                        span { "{estimate_reading_time(&post.content)} min" }
                                                    }

                                                    if post.view_count > 0 {
                                                        span { class: "text-border", "•" }
                                                        div { class: "flex items-center gap-1",
                                                            Icon { icon: LdEye, class: "w-3 h-3" }
                                                            span { "{post.view_count}" }
                                                        }
                                                    }
                                                }
                                            }

                                            // Thumbnail
                                            if let Some(img) = &post.featured_image {
                                                div { class: "w-full md:w-48 lg:w-56 flex-shrink-0",
                                                    div { class: "aspect-video md:aspect-[4/3] rounded-lg overflow-hidden bg-muted/30",
                                                        img {
                                                            src: "{img.file_url}",
                                                            alt: "{post.title}",
                                                            class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "text-muted-foreground", "No content available" }
                    }
                }
            }
        }
    }
}
