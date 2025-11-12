use dioxus::prelude::*;

/// Alert component variants, similar to shadcn/ui Alert
#[derive(PartialEq, Clone, Copy)]
pub enum AlertVariant {
    Default,
    Destructive,
}

/// Properties for the Alert component
#[derive(Props, PartialEq, Clone)]
pub struct AlertProps {
    /// The content to be displayed inside the alert
    children: Element,

    /// Additional CSS classes to apply to the alert
    #[props(default)]
    class: Option<String>,

    /// The variant of the alert
    #[props(default = AlertVariant::Default)]
    variant: AlertVariant,
}

/// Properties for the AlertTitle component
#[derive(Props, PartialEq, Clone)]
pub struct AlertTitleProps {
    /// The content to be displayed inside the alert title
    children: Element,

    /// Additional CSS classes to apply to the title
    #[props(default)]
    class: Option<String>,
}

/// Properties for the AlertDescription component
#[derive(Props, PartialEq, Clone)]
pub struct AlertDescriptionProps {
    /// The content to be displayed inside the alert description
    children: Element,

    /// Additional CSS classes to apply to the description
    #[props(default)]
    class: Option<String>,
}

/// Get the CSS class for an alert variant
fn get_variant_class(variant: AlertVariant) -> &'static str {
    match variant {
        AlertVariant::Default => "bg-card text-card-foreground",
        AlertVariant::Destructive => "text-destructive bg-card [&>svg]:text-current *:data-[slot=alert-description]:text-destructive/90",
    }
}

/// Alert component
#[component]
pub fn Alert(props: AlertProps) -> Element {
    // Combine all CSS classes
    let mut class = vec![
        "relative w-full rounded-lg border px-4 py-3 text-sm grid has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] grid-cols-[0_1fr] has-[>svg]:gap-x-3 gap-y-0.5 items-start [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current".to_string(),
        get_variant_class(props.variant).to_string(),
    ];

    // Add custom class if provided
    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { role: "alert", class: class.join(" "), {props.children} }
    }
}

/// AlertTitle component
#[component]
pub fn AlertTitle(props: AlertTitleProps) -> Element {
    let mut class = vec!["col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight".to_string()];

    // Add custom class if provided
    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        div { class: class.join(" "), {props.children} }
    }
}

/// AlertDescription component
#[component]
pub fn AlertDescription(props: AlertDescriptionProps) -> Element {
    let mut class = vec![
        "text-muted-foreground col-start-2 grid justify-items-start gap-1 text-sm [&_p]:leading-relaxed".to_string(),
    ];

    // Add custom class if provided
    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        div { class: class.join(" "), {props.children} }
    }
}
