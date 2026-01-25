use dioxus::prelude::*;
use ruxlog_shared::store::categories::Category;

#[derive(Props, Clone, PartialEq)]
pub struct CategoryCardProps {
    pub category: Category,
    #[props(into)]
    pub on_click: Option<EventHandler<String>>,
}

#[component]
pub fn CategoryCard(props: CategoryCardProps) -> Element {
    let category = props.category.clone();
    let category_slug = category.slug.clone();

    rsx! {
        article {
            class: "category-card group h-full",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(category_slug.clone());
                }
            },

            // Media (only if cover exists)
            if let Some(cover) = &category.cover {
                div { class: "category-card-image",
                    img {
                        src: "{cover.file_url}",
                        alt: "{category.name}",
                    }

                    // Logo badge
                    if let Some(logo) = &category.logo {
                        div { class: "category-logo-badge",
                            img {
                                src: "{logo.file_url}",
                                alt: "{category.name}",
                                class: "w-full h-full object-contain",
                            }
                        }
                    }
                }
            } else {
                // Fallback gradient if no cover
                div { class: "aspect-[16/9] bg-gradient-to-br from-cyan-500/10 via-violet-500/10 to-purple-500/10" }
            }

            // Content
            div { class: "p-5",
                // Category label
                span { class: "section-label mb-3 inline-block",
                    "Category"
                }

                h3 { class: "text-xl font-bold mb-2 line-clamp-2 group-hover:text-violet-600 dark:group-hover:text-violet-400 transition-colors",
                    "{category.name}"
                }

                if let Some(description) = &category.description {
                    p { class: "text-muted-foreground text-sm leading-relaxed line-clamp-2",
                        "{description}"
                    }
                }
            }
        }
    }
}
