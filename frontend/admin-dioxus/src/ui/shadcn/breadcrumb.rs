use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdChevronRight, LdEllipsis},
    Icon,
};

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbListProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbItemProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbLinkProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub onclick: Option<Callback<()>>,
}

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbPageProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbSeparatorProps {
    #[props(default)]
    pub children: Option<Element>,
    #[props(default)]
    pub class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct BreadcrumbEllipsisProps {
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn Breadcrumb(props: BreadcrumbProps) -> Element {
    let class_str = props.class.clone().unwrap_or_default();

    rsx! {
        nav {
            "data-slot": "breadcrumb",
            "aria-label": "breadcrumb",
            class: class_str,
            {props.children}
        }
    }
}

#[component]
pub fn BreadcrumbList(props: BreadcrumbListProps) -> Element {
    let mut class = vec![
        "text-muted-foreground flex flex-wrap items-center gap-1.5 text-sm break-words sm:gap-2.5"
            .to_string(),
    ];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        ol { "data-slot": "breadcrumb-list", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn BreadcrumbItem(props: BreadcrumbItemProps) -> Element {
    let mut class = vec!["inline-flex items-center gap-1.5 cursor-pointer".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        li { "data-slot": "breadcrumb-item", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn BreadcrumbLink(props: BreadcrumbLinkProps) -> Element {
    let mut class = vec!["hover:text-foreground transition-colors".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        a {
            "data-slot": "breadcrumb-link",
            class: class.join(" "),
            href: props.href,
            onclick: move |_| {
                if let Some(handler) = &props.onclick {
                    handler.call(())
                }
            },
            {props.children}
        }
    }
}

#[component]
pub fn BreadcrumbPage(props: BreadcrumbPageProps) -> Element {
    let mut class = vec!["text-foreground font-normal".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        span {
            "data-slot": "breadcrumb-page",
            role: "link",
            "aria-disabled": "true",
            "aria-current": "page",
            class: class.join(" "),
            {props.children}
        }
    }
}

#[component]
pub fn BreadcrumbSeparator(props: BreadcrumbSeparatorProps) -> Element {
    let mut class = vec!["[&>svg]:size-3.5".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    // Using match for conditional rendering based on props.children
    match &props.children {
        Some(children) => {
            rsx! {
                li {
                    "data-slot": "breadcrumb-separator",
                    role: "presentation",
                    "aria-hidden": "true",
                    class: class.join(" "),
                    {children}
                }
            }
        }
        None => {
            rsx! {
                li {
                    "data-slot": "breadcrumb-separator",
                    role: "presentation",
                    "aria-hidden": "true",
                    class: class.join(" "),
                    Icon { icon: LdChevronRight }
                }
            }
        }
    }
}

#[component]
pub fn BreadcrumbEllipsis(props: BreadcrumbEllipsisProps) -> Element {
    let mut class = vec!["flex size-9 items-center justify-center".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        span {
            "data-slot": "breadcrumb-ellipsis",
            role: "presentation",
            "aria-hidden": "true",
            class: class.join(" "),
            Icon { icon: LdEllipsis }
            span { class: "sr-only", "More" }
        }
    }
}
