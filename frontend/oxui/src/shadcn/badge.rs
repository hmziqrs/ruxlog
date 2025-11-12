use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct BadgeProps {
    #[props(optional)]
    pub class: String,

    pub children: Element,

    #[props(default = BadgeVariant::Default)]
    pub variant: BadgeVariant,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[derive(PartialEq, Clone)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

impl BadgeVariant {
    fn to_class_string(&self) -> &'static str {
        match self {
            BadgeVariant::Default => "border-transparent bg-primary text-primary-foreground [a&]:hover:bg-primary/90",
            BadgeVariant::Secondary => "border-transparent bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90",
            BadgeVariant::Destructive => "border-transparent bg-destructive text-white [a&]:hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60",
            BadgeVariant::Outline => "text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
        }
    }
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let base_classes = "inline-flex items-center justify-center rounded-4xl border px-2 py-0.5 text-xs font-semibold w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 gap-1 [&>svg]:pointer-events-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive transition-[color,box-shadow] overflow-hidden";

    let classes = vec![
        base_classes.to_string(),
        props.variant.to_class_string().to_string(),
        props.class.clone(),
    ];

    rsx! {
        div {
            "data-slot": "badge",
            class: classes.join(" "),
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}
