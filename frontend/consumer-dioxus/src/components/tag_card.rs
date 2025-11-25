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
            class: "group h-full rounded-lg border border-border overflow-hidden transition-colors duration-200 hover:border-primary/50 cursor-pointer",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(tag_slug.clone());
                }
            },

            div { class: "p-4",
                // Color indicator
                div { class: "mb-3",
                    div {
                        class: "inline-block w-3 h-3 rounded-full",
                        style: "background-color: {tag.color};",
                    }
                }

                h3 { class: "text-lg font-semibold mb-2 leading-snug group-hover:text-primary transition-colors",
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
