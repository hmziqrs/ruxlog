use dioxus::prelude::*;

/// Properties for the Skeleton component
#[derive(Props, PartialEq, Clone)]
pub struct SkeletonProps {
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
    /// Optional child elements to render inside the skeleton
    #[props(default)]
    pub children: Option<Element>,
}

/// Skeleton component for loading placeholders
#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let mut class = vec!["bg-accent animate-pulse rounded-md".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        div {
            "data-slot": "skeleton",
            class: class.join(" "),
            if let Some(children) = &props.children {
                {children}
            }
        }
    }
}
