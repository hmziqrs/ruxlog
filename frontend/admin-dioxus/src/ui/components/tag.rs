use dioxus::prelude::*;

use crate::store::Tag;

#[derive(PartialEq, Clone, Copy)]
pub enum TagSize {
    Sm,
    Md,
    Lg,
}

fn size_class(size: TagSize) -> &'static str {
    match size {
        TagSize::Sm => "px-2 py-1 text-xs",
        TagSize::Md => "px-2.5 py-1.5 text-sm",
        TagSize::Lg => "px-3 py-2 text-base",
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct TagBadgeProps {
    pub tag: Tag,
    #[props(default = TagSize::Md)]
    pub size: TagSize,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A reusable tag badge/pill component used across the app.
#[component]
pub fn TagBadge(props: TagBadgeProps) -> Element {
    let mut classes = vec![
        "inline-flex items-center rounded-md font-medium shadow-sm ring-1 ring-inset",
        // default ring color is subtle and works on light/dark backgrounds
        "ring-black/5 dark:ring-white/10",
        size_class(props.size),
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

    if let Some(c) = &props.class {
        classes.push(c.clone());
    }

    // Inline style for dynamic background and text color
    let style = format!(
        "background-color: {}; color: {}; border-color: rgba(0,0,0,0.06);",
        props.tag.color, props.tag.text_color
    );

    rsx! {
        span { class: classes.join(" "), style: style, ..props.attributes,
            span { class: "mr-2 inline-block w-2.5 h-2.5 rounded-full", style: "background-color: rgba(255,255,255,0.4);" }
            { props.tag.name.clone() }
        }
    }
}
