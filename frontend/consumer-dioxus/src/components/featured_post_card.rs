use dioxus::prelude::*;
use ruxlog_shared::store::posts::Post;
use hmziq_dioxus_free_icons::icons::ld_icons::LdArrowRight;
use hmziq_dioxus_free_icons::Icon;
use super::post_card::{estimate_reading_time, format_date};

#[derive(Props, Clone, PartialEq)]
pub struct FeaturedPostCardProps {
    pub post: Post,
    #[props(into)]
    pub on_click: Option<EventHandler<i32>>,
}

/// Hero-style featured post card
#[component]
pub fn FeaturedPostCard(props: FeaturedPostCardProps) -> Element {
    let post = props.post.clone();
    let post_id = post.id;

    rsx! {
        article {
            class: "group cursor-pointer rounded-lg border border-border overflow-hidden transition-colors duration-200 hover:border-primary/50",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(post_id);
                }
            },
            // Media section
            div { class: "relative aspect-[21/9] overflow-hidden bg-muted",
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

                // Category badge - top left
                div { class: "absolute top-4 left-4",
                    span { class: "px-2.5 py-1 text-xs font-medium border border-border rounded bg-background",
                        "{post.category.name}"
                    }
                }
            }

            // Content
            div { class: "p-6",
                // Tags
                if !post.tags.is_empty() {
                    div { class: "flex flex-wrap gap-2 mb-3",
                        for tag in post.tags.iter().take(3) {
                            span { class: "text-sm",
                                "{tag.name}"
                            }
                        }
                    }
                }

                h2 { class: "text-2xl md:text-3xl font-bold mb-3 leading-tight",
                    "{post.title}"
                }

                if let Some(excerpt) = &post.excerpt {
                    p { class: "text-base leading-relaxed mb-4 line-clamp-2",
                        "{excerpt}"
                    }
                }

                // Meta row
                div { class: "flex flex-wrap items-center justify-between gap-4",
                    div { class: "flex items-center gap-2 text-sm",
                        span { "{post.author.name}" }
                        span { "·" }
                        if let Some(published) = &post.published_at {
                            span { "{format_date(published)}" }
                        }
                        span { "·" }
                        span { "{estimate_reading_time(&post.content)} min read" }
                    }

                    // CTA
                    div { class: "flex items-center gap-2 font-medium text-sm",
                        span { "Read article" }
                        Icon { icon: LdArrowRight, class: "w-4 h-4" }
                    }
                }
            }
        }
    }
}
