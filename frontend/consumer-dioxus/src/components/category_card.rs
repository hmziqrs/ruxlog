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
            class: "group h-full rounded-lg border border-border overflow-hidden transition-colors duration-200 hover:border-primary/50 cursor-pointer",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(category_slug.clone());
                }
            },

            // Media (only if cover exists)
            if let Some(cover) = &category.cover {
                div { class: "relative aspect-[16/9] overflow-hidden bg-muted",
                    img {
                        src: "{cover.file_url}",
                        alt: "{category.name}",
                        class: "w-full h-full object-cover transition-transform duration-500 group-hover:scale-105",
                    }

                    // Logo badge
                    if let Some(logo) = &category.logo {
                        div { class: "absolute top-3 left-3",
                            div { class: "w-8 h-8 border border-border rounded bg-background flex items-center justify-center p-1",
                                img {
                                    src: "{logo.file_url}",
                                    alt: "{category.name}",
                                    class: "w-full h-full object-contain",
                                }
                            }
                        }
                    }
                }
            }

            // Content
            div { class: "p-4",
                h3 { class: "text-lg font-semibold mb-2 leading-snug line-clamp-2",
                    "{category.name}"
                }

                if let Some(description) = &category.description {
                    p { class: "text-sm leading-relaxed mb-3 line-clamp-2",
                        "{description}"
                    }
                }
            }
        }
    }
}
