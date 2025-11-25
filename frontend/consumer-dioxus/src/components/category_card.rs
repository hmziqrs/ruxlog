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

            // Media section
            div { class: "relative aspect-[16/9] overflow-hidden bg-muted",
                if let Some(cover) = &category.cover {
                    img {
                        src: "{cover.file_url}",
                        alt: "{category.name}",
                        class: "w-full h-full object-cover transition-transform duration-500 group-hover:scale-105",
                    }
                } else {
                    div {
                        class: "w-full h-full bg-muted",
                        style: "background: linear-gradient(135deg, {category.color}20 0%, transparent 100%);",
                    }
                }

                // Logo overlay
                if let Some(logo) = &category.logo {
                    div { class: "absolute top-3 left-3",
                        img {
                            src: "{logo.file_url}",
                            alt: "{category.name}",
                            class: "w-10 h-10 rounded bg-background/90 p-1.5 object-contain",
                        }
                    }
                }
            }

            // Content
            div { class: "p-4",
                h3 { class: "text-lg font-semibold mb-2 leading-snug group-hover:text-primary transition-colors",
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
