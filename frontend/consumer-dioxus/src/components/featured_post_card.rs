use dioxus::prelude::*;
use ruxlog_shared::store::posts::Post;
use hmziq_dioxus_free_icons::icons::ld_icons::LdArrowRight;
use hmziq_dioxus_free_icons::Icon;
use super::post_card::{estimate_reading_time, format_date, get_gradient_for_tag};

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
    let first_tag = post.tags.first().map(|t| t.name.clone());
    let first_tag_str = first_tag.as_deref();
    let gradient = get_gradient_for_tag(first_tag_str);

    rsx! {
        article {
            class: "group cursor-pointer",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(post_id);
                }
            },
            // Gradient border wrapper
            div { class: "rounded-2xl bg-gradient-to-br from-primary/30 via-primary/10 to-transparent p-[1px] transition-all duration-500 group-hover:from-primary/50 group-hover:via-primary/20",
                div { class: "rounded-2xl bg-card/95 backdrop-blur-sm overflow-hidden",
                    // Media section
                    div { class: "relative aspect-[21/9] overflow-hidden",
                        if let Some(img) = &post.featured_image {
                            img {
                                src: "{img.file_url}",
                                alt: "{post.title}",
                                class: "w-full h-full object-cover transition-transform duration-700 group-hover:scale-105",
                            }
                            // Overlay gradient
                            div { class: "absolute inset-0 bg-gradient-to-t from-card via-card/20 to-transparent" }
                        } else {
                            // Fallback gradient based on tag
                            div {
                                class: "w-full h-full bg-gradient-to-br {gradient}",
                                div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-primary/10 via-transparent to-transparent" }
                                // Pattern overlay
                                div {
                                    class: "absolute inset-0 opacity-[0.03]",
                                    style: "background-image: url(\"data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E\");",
                                }
                            }
                        }

                        // Category tags - top left
                        if !post.tags.is_empty() {
                            div { class: "absolute top-4 left-4 flex flex-wrap gap-2",
                                for tag in post.tags.iter().take(2) {
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
                            "{post.title}"
                        }

                        if let Some(excerpt) = &post.excerpt {
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
                                        "{post.author.name.chars().next().unwrap_or('A').to_uppercase()}"
                                    }
                                    span { class: "font-medium text-foreground", "{post.author.name}" }
                                }
                                span { class: "text-border", "·" }
                                if let Some(published) = &post.published_at {
                                    span { "{format_date(published)}" }
                                }
                                span { class: "text-border", "·" }
                                span { "{estimate_reading_time(&post.content)} min read" }
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
}
