use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::LdBookOpen;
use hmziq_dioxus_free_icons::Icon;

/// Empty state for posts list
#[component]
pub fn PostsEmptyState() -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center py-24 text-center",
            div { class: "w-20 h-20 rounded-2xl bg-gradient-to-br from-primary/20 to-primary/5 flex items-center justify-center mb-6",
                Icon { icon: LdBookOpen, class: "w-10 h-10 text-primary" }
            }
            h3 { class: "text-2xl font-bold mb-3", "No posts yet" }
            p { class: "text-muted-foreground max-w-md text-lg",
                "The first article is on its way. Check back soon."
            }
        }
    }
}

/// Loading skeleton for posts
#[component]
pub fn PostsLoadingSkeleton() -> Element {
    rsx! {
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
    }
}

/// Loading skeleton for single post card
#[component]
pub fn PostCardSkeleton() -> Element {
    rsx! {
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
