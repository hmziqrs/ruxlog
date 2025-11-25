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
    
    rsx! {
        article {
            class: "group h-full rounded-lg border border-border overflow-hidden transition-colors duration-200 hover:border-primary/50 cursor-pointer",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(post_id);
                }
            },
            // Media
            div { class: "relative aspect-[16/9] overflow-hidden bg-muted",
                if let Some(img) = &post.featured_image {
                    img {
                        src: "{img.file_url}",
                        alt: "{post.title}",
                        class: "w-full h-full object-cover transition-transform duration-500 group-hover:scale-105",
                    }
                } else {
                    // Fallback - simple muted background
                    div { class: "w-full h-full bg-muted" }
                }

                // Category badge
                div { class: "absolute top-3 left-3",
                    span { class: "px-2 py-1 text-xs font-medium border border-border rounded bg-background",
                        "{post.category.name}"
                    }
                }
            }

            // Content
            div { class: "p-4",
                // Tags
                if !post.tags.is_empty() {
                    div { class: "flex flex-wrap gap-2 mb-2",
                        for tag in post.tags.iter().take(2) {
                            span { class: "text-xs",
                                "{tag.name}"
                            }
                        }
                    }
                }

                h3 { class: "text-lg font-semibold mb-2 leading-snug line-clamp-2",
                    "{post.title}"
                }

                if let Some(excerpt) = &post.excerpt {
                    p { class: "text-sm leading-relaxed mb-3 line-clamp-2",
                        "{excerpt}"
                    }
                }

                // Meta
                div { class: "flex items-center gap-2 text-xs",
                    span { "{post.author.name}" }
                    span { "·" }
                    if let Some(published) = &post.published_at {
                        span { "{format_date(published)}" }
                    }
                    span { "·" }
                    span { "{estimate_reading_time(&post.content)} min" }
                }
            }
        }
    }
}
