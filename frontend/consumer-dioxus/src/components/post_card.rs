use dioxus::prelude::*;
use ruxlog_shared::store::posts::{Post, PostContent};
use chrono::{DateTime, Utc};

/// Estimate reading time based on content blocks (avg 200 words per minute)
pub fn estimate_reading_time(content: &PostContent) -> u32 {
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
pub fn format_date(date: &DateTime<Utc>) -> String {
    date.format("%b %d, %Y").to_string()
}

/// Generate a gradient class based on tag name for fallback backgrounds
pub fn get_gradient_for_tag(tag: Option<&str>) -> &'static str {
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

#[derive(Props, Clone, PartialEq)]
pub struct PostCardProps {
    pub post: Post,
    #[props(into)]
    pub on_click: Option<EventHandler<i32>>,
}

/// Standard post card for grid layout
#[component]
pub fn PostCard(props: PostCardProps) -> Element {
    let post = props.post.clone();
    let post_id = post.id;
    let first_tag = post.tags.first().map(|t| t.name.clone());
    let first_tag_str = first_tag.as_deref();
    let gradient = get_gradient_for_tag(first_tag_str);
    
    rsx! {
        article {
            class: "group h-full rounded-xl border border-border/50 bg-card/50 backdrop-blur-sm overflow-hidden transition-all duration-300 hover:border-primary/30 hover:bg-card/80 hover:shadow-lg hover:shadow-primary/5 cursor-pointer",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(post_id);
                }
            },
            // Media
            div { class: "relative aspect-[16/9] overflow-hidden",
                if let Some(img) = &post.featured_image {
                    img {
                        src: "{img.file_url}",
                        alt: "{post.title}",
                        class: "w-full h-full object-cover transition-transform duration-500 group-hover:scale-105",
                    }
                } else {
                    // Fallback gradient
                    div {
                        class: "w-full h-full bg-gradient-to-br {gradient}",
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
