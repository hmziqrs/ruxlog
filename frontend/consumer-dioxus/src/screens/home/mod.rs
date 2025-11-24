use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use ruxlog_shared::store::use_post;
use ruxlog_shared::store::posts::PostContent;
use crate::router::Route;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdArrowRight, LdBookOpen};
use hmziq_dioxus_free_icons::Icon;
use chrono::{DateTime, Utc};

/// Estimate reading time based on content blocks (avg 200 words per minute)
fn estimate_reading_time(content: &PostContent) -> u32 {
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

/// Generate a gradient class based on tag name for fallback backgrounds
fn get_gradient_for_tag(tag: Option<&str>) -> &'static str {
    match tag.unwrap_or("").to_lowercase().as_str() {
        s if s.contains("rust") => "from-orange-500/20 via-red-500/10 to-transparent",
        s if s.contains("react") || s.contains("frontend") => "from-cyan-500/20 via-blue-500/10 to-transparent",
        s if s.contains("backend") || s.contains("api") => "from-green-500/20 via-emerald-500/10 to-transparent",
        s if s.contains("devops") || s.contains("infra") => "from-purple-500/20 via-violet-500/10 to-transparent",
        s if s.contains("database") || s.contains("sql") => "from-yellow-500/20 via-amber-500/10 to-transparent",
        s if s.contains("security") => "from-red-500/20 via-rose-500/10 to-transparent",
        s if s.contains("ai") || s.contains("ml") => "from-pink-500/20 via-fuchsia-500/10 to-transparent",
        _ => "from-primary/20 via-primary/5 to-transparent",
    }
}

#[component]
pub fn HomeScreen() -> Element {
    let posts_store = use_post();

    use_effect(move || {
        let posts = posts_store;
        spawn(async move {
            posts.list().await;
        });
    });

    let posts_frame = posts_store.list.read();

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-6xl",
                if (*posts_frame).is_loading() {
                    // Loading skeleton
                    div { class: "space-y-8",
                        // Featured skeleton
                        div { class: "animate-pulse",
                            div { class: "rounded-2xl bg-gradient-to-br from-muted/50 to-muted/20 p-1",
                                div { class: "rounded-xl bg-card/80 overflow-hidden",
                                    div { class: "aspect-[21/9] bg-muted/40" }
                                    div { class: "p-6 space-y-4",
                                        div { class: "flex gap-2",
                                            div { class: "h-6 w-16 bg-muted/50 rounded-full" }
                                            div { class: "h-6 w-20 bg-muted/50 rounded-full" }
                                        }
                                        div { class: "h-8 bg-muted/50 rounded w-3/4" }
                                        div { class: "h-4 bg-muted/40 rounded w-full" }
                                        div { class: "h-4 bg-muted/40 rounded w-2/3" }
                                    }
                                }
                            }
                        }
                        // Grid skeleton
                        div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for _ in 0..3 {
                                div { class: "animate-pulse",
                                    div { class: "rounded-xl border border-border/40 bg-card/50 overflow-hidden",
                                        div { class: "aspect-[16/9] bg-muted/40" }
                                        div { class: "p-5 space-y-3",
                                            div { class: "h-5 w-16 bg-muted/50 rounded-full" }
                                            div { class: "h-6 bg-muted/50 rounded w-full" }
                                            div { class: "h-4 bg-muted/40 rounded w-3/4" }
                                        }
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
                        div { class: "flex flex-col items-center justify-center py-24 text-center",
                            div { class: "w-20 h-20 rounded-2xl bg-gradient-to-br from-primary/20 to-primary/5 flex items-center justify-center mb-6",
                                Icon { icon: LdBookOpen, class: "w-10 h-10 text-primary" }
                            }
                            h3 { class: "text-2xl font-bold mb-3", "No posts yet" }
                            p { class: "text-muted-foreground max-w-md text-lg",
                                "The first article is on its way. Check back soon."
                            }
                        }
                    } else {
                        div { class: "space-y-10",
                            // Featured post (hero card)
                            if let Some(featured) = data.data.first() {
                                Link {
                                    to: Route::PostViewScreen { id: featured.id },
                                    class: "group block",
                                    // Gradient border wrapper
                                    div { class: "rounded-2xl bg-gradient-to-br from-primary/30 via-primary/10 to-transparent p-[1px] transition-all duration-500 group-hover:from-primary/50 group-hover:via-primary/20",
                                        div { class: "rounded-2xl bg-card/95 backdrop-blur-sm overflow-hidden",
                                            // Media section
                                            div { class: "relative aspect-[21/9] overflow-hidden",
                                                if let Some(img) = &featured.featured_image {
                                                    img {
                                                        src: "{img.file_url}",
                                                        alt: "{featured.title}",
                                                        class: "w-full h-full object-cover transition-transform duration-700 group-hover:scale-105",
                                                    }
                                                    // Overlay gradient
                                                    div { class: "absolute inset-0 bg-gradient-to-t from-card via-card/20 to-transparent" }
                                                } else {
                                                    // Fallback gradient based on tag
                                                    div {
                                                        class: "w-full h-full bg-gradient-to-br {get_gradient_for_tag(featured.tags.first().map(|t| t.name.as_str()))}",
                                                        div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-primary/10 via-transparent to-transparent" }
                                                        // Pattern overlay
                                                        div {
                                                            class: "absolute inset-0 opacity-[0.03]",
                                                            style: "background-image: url(\"data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E\");",
                                                        }
                                                    }
                                                }

                                                // Category tags - top left
                                                if !featured.tags.is_empty() {
                                                    div { class: "absolute top-4 left-4 flex flex-wrap gap-2",
                                                        for tag in featured.tags.iter().take(2) {
                                                            span { class: "px-3 py-1.5 rounded-full text-xs font-semibold bg-background/90 backdrop-blur-sm text-foreground border border-border/50 shadow-sm",
                                                                "{tag.name}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Content
                                            div { class: "p-6 md:p-8 -mt-16 relative z-10",
                                                h2 { class: "text-2xl md:text-3xl lg:text-4xl font-bold mb-3 leading-tight group-hover:text-primary transition-colors duration-300",
                                                    "{featured.title}"
                                                }

                                                if let Some(excerpt) = &featured.excerpt {
                                                    p { class: "text-muted-foreground text-base md:text-lg leading-relaxed mb-5 line-clamp-2",
                                                        "{excerpt}"
                                                    }
                                                }

                                                // Meta row
                                                div { class: "flex flex-wrap items-center justify-between gap-4",
                                                    div { class: "flex items-center gap-4 text-sm text-muted-foreground",
                                                        // Author
                                                        div { class: "flex items-center gap-2",
                                                            div { class: "w-8 h-8 rounded-full bg-gradient-to-br from-primary/30 to-primary/10 flex items-center justify-center text-sm font-bold text-primary",
                                                                "{featured.author.name.chars().next().unwrap_or('A').to_uppercase()}"
                                                            }
                                                            span { class: "font-medium text-foreground", "{featured.author.name}" }
                                                        }
                                                        span { class: "text-border", "·" }
                                                        if let Some(published) = &featured.published_at {
                                                            span { "{format_date(published)}" }
                                                        }
                                                        span { class: "text-border", "·" }
                                                        span { "{estimate_reading_time(&featured.content)} min read" }
                                                    }

                                                    // CTA
                                                    div { class: "flex items-center gap-2 text-primary font-semibold text-sm group-hover:gap-3 transition-all duration-300",
                                                        span { "Read article" }
                                                        Icon { icon: LdArrowRight, class: "w-4 h-4" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Posts grid
                            if data.data.len() > 1 {
                                div { class: "grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                                    for post in data.data.iter().skip(1) {
                                        Link {
                                            to: Route::PostViewScreen { id: post.id },
                                            class: "group block",
                                            article { class: "h-full rounded-xl border border-border/50 bg-card/50 backdrop-blur-sm overflow-hidden transition-all duration-300 hover:border-primary/30 hover:bg-card/80 hover:shadow-lg hover:shadow-primary/5",
                                                // Media
                                                div { class: "relative aspect-[16/9] overflow-hidden",
                                                    if let Some(img) = &post.featured_image {
                                                        img {
                                                            src: "{img.file_url}",
                                                            alt: "{post.title}",
                                                            class: "w-full h-full object-cover transition-transform duration-500 group-hover:scale-105",
                                                        }
                                                    } else {
                                                        // Fallback
                                                        div {
                                                            class: "w-full h-full bg-gradient-to-br {get_gradient_for_tag(post.tags.first().map(|t| t.name.as_str()))}",
                                                            div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_left,_var(--tw-gradient-stops))] from-primary/5 via-transparent to-transparent" }
                                                        }
                                                    }

                                                    // Tag badge
                                                    if let Some(tag) = post.tags.first() {
                                                        div { class: "absolute top-3 left-3",
                                                            span { class: "px-2.5 py-1 rounded-full text-xs font-semibold bg-background/90 backdrop-blur-sm text-foreground border border-border/50",
                                                                "{tag.name}"
                                                            }
                                                        }
                                                    }
                                                }

                                                // Content
                                                div { class: "p-5",
                                                    h3 { class: "text-lg font-bold mb-2 leading-snug group-hover:text-primary transition-colors duration-300 line-clamp-2",
                                                        "{post.title}"
                                                    }

                                                    if let Some(excerpt) = &post.excerpt {
                                                        p { class: "text-muted-foreground text-sm leading-relaxed mb-4 line-clamp-2",
                                                            "{excerpt}"
                                                        }
                                                    }

                                                    // Meta
                                                    div { class: "flex items-center gap-3 text-xs text-muted-foreground",
                                                        div { class: "flex items-center gap-1.5",
                                                            div { class: "w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center text-[10px] font-bold text-primary",
                                                                "{post.author.name.chars().next().unwrap_or('A').to_uppercase()}"
                                                            }
                                                            span { "{post.author.name}" }
                                                        }
                                                        span { class: "text-border", "·" }
                                                        if let Some(published) = &post.published_at {
                                                            span { "{format_date(published)}" }
                                                        }
                                                        span { class: "text-border", "·" }
                                                        span { "{estimate_reading_time(&post.content)} min" }
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
