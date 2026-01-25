use dioxus::prelude::*;
use ruxlog_shared::store::tags::Tag;

#[derive(Props, Clone, PartialEq)]
pub struct TagCardProps {
    pub tag: Tag,
    #[props(into)]
    pub on_click: Option<EventHandler<String>>,
}

#[component]
pub fn TagCard(props: TagCardProps) -> Element {
    let tag = props.tag.clone();
    let tag_slug = tag.slug.clone();

    rsx! {
        article {
            class: "tag-card group h-full",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(tag_slug.clone());
                }
            },

            div { class: "p-5",
                // Color indicator with ring
                div { class: "flex items-center gap-3 mb-4",
                    div {
                        class: "tag-color-dot",
                        style: "background-color: {tag.color}; --tw-ring-color: {tag.color}40;",
                    }
                    span { class: "section-label",
                        "Tag"
                    }
                }

                h3 { class: "text-xl font-bold mb-2 group-hover:text-violet-600 dark:group-hover:text-violet-400 transition-colors",
                    "{tag.name}"
                }

                if let Some(description) = &tag.description {
                    p { class: "text-muted-foreground text-sm leading-relaxed line-clamp-2",
                        "{description}"
                    }
                }
            }
        }
    }
}
