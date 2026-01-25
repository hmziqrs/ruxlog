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
            class: "card-featured group cursor-pointer",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(post_id);
                }
            },
            // Media section
            div { class: "relative aspect-[21/9] overflow-hidden",
                if let Some(img) = &post.featured_image {
                    img {
                        src: "{img.file_url}",
                        alt: "{post.title}",
                        class: "w-full h-full object-cover transition-transform duration-700 group-hover:scale-105",
                    }
                } else {
                    // Fallback - vibrant gradient
                    div { class: "w-full h-full bg-gradient-to-br from-violet-500/20 via-purple-500/10 to-cyan-500/20" }
                }

                // Gradient overlay for better text contrast
                div { class: "absolute inset-0 bg-gradient-to-t from-black/20 to-transparent" }

                // Category badge - top left
                div { class: "absolute top-4 left-4",
                    span { class: "category-pill",
                        "{post.category.name}"
                    }
                }

                // Featured badge - top right
                div { class: "absolute top-4 right-4",
                    span { class: "inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-gradient-to-r from-amber-500 to-orange-500 text-white shadow-lg",
                        "Featured"
                    }
                }
            }

            // Content
            div { class: "p-6 md:p-8",
                // Tags - colorful chips
                if !post.tags.is_empty() {
                    div { class: "flex flex-wrap gap-2 mb-4",
                        for tag in post.tags.iter().take(3) {
                            span { class: "tag-chip",
                                "{tag.name}"
                            }
                        }
                    }
                }

                h2 { class: "text-2xl md:text-3xl font-extrabold leading-tight tracking-tight mb-4 group-hover:text-violet-600 dark:group-hover:text-violet-400 transition-colors",
                    "{post.title}"
                }

                if let Some(excerpt) = &post.excerpt {
                    p { class: "text-base md:text-lg text-muted-foreground leading-relaxed mb-6 line-clamp-2",
                        "{excerpt}"
                    }
                }

                // Meta row
                div { class: "flex flex-wrap items-center justify-between gap-4",
                    div { class: "meta-row",
                        span { class: "meta-author", "{post.author.name}" }
                        span { class: "meta-dot" }
                        if let Some(published) = &post.published_at {
                            span { class: "meta-date", "{format_date(published)}" }
                        }
                        span { class: "meta-dot" }
                        span { class: "meta-reading-time", "{estimate_reading_time(&post.content)} min read" }
                    }

                    // CTA Button
                    div { class: "cta-button",
                        span { "Read article" }
                        Icon { icon: LdArrowRight, class: "w-4 h-4" }
                    }
                }
            }
        }
    }
}
