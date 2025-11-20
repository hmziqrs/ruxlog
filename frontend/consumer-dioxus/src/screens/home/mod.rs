use dioxus::prelude::*;
use ruxlog_shared::store::use_post;

#[component]
pub fn HomeScreen() -> Element {
    let posts_store = use_post();

    // Fetch posts on mount
    use_effect(move || {
        let posts = posts_store;c
        spawn(async move {
            posts.list().await;
        });
    });

    let posts_frame = posts_store.list.read();

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            // Header
            div { class: "border-b border-border/60 backdrop-blur-xl",
                div { class: "container mx-auto px-4 py-8",
                    h1 { class: "text-4xl font-bold mb-2", "Welcome to Ruxlog" }
                    p { class: "text-muted-foreground",
                        "Discover stories, thinking, and expertise from writers on any topic."
                    }
                }
            }

            // Posts grid
            div { class: "container mx-auto px-4 py-8",
                if (*posts_frame).is_loading() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "text-muted-foreground", "Loading posts..." }
                    }
                } else if let Some(error) = (*posts_frame).error_message() {
                    div { class: "flex items-center justify-center py-20",
                        div { class: "text-destructive", "Error: {error}" }
                    }
                } else if let Some(data) = &(*posts_frame).data {
                    if data.data.is_empty() {
                        div { class: "flex items-center justify-center py-20",
                            div { class: "text-muted-foreground", "No posts found" }
                        }
                    } else {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for post in &data.data {
                                a {
                                    href: "/posts/{post.id}",
                                    class: "group block rounded-lg border border-border bg-card text-card-foreground shadow-sm hover:shadow-md transition-shadow",
                                    if let Some(featured_image) = &post.featured_image {
                                        div { class: "aspect-video w-full overflow-hidden rounded-t-lg",
                                            img {
                                                src: "{featured_image.file_url}",
                                                alt: "{post.title}",
                                                class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300",
                                            }
                                        }
                                    }

                                    div { class: "p-6",
                                        h2 { class: "text-xl font-semibold mb-2 group-hover:text-primary transition-colors",
                                            "{post.title}"
                                        }
                                        if let Some(excerpt) = &post.excerpt {
                                            p { class: "text-muted-foreground text-sm mb-4 line-clamp-3",
                                                "{excerpt}"
                                            }
                                        }

                                        div { class: "flex items-center gap-4 text-xs text-muted-foreground",
                                            span { "By {post.author.name}" }
                                            span { "•" }
                                            span { "{post.view_count} views" }
                                        }

                                        if !post.tags.is_empty() {
                                            div { class: "flex flex-wrap gap-2 mt-4",
                                                for tag in &post.tags {
                                                    span { class: "px-2 py-1 rounded-full text-xs bg-muted",
                                                        "{tag.name}"
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
