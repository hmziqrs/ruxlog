use dioxus::prelude::*;

/// Properties for Card and its sub-components
#[derive(Props, PartialEq, Clone)]
pub struct CardProps {
    /// The content to be displayed inside the card
    children: Element,
    /// Additional CSS classes to apply to the card
    #[props(default)]
    class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct CardHeaderProps {
    /// The content to be displayed inside the card header
    children: Element,
    /// Additional CSS classes to apply to the header
    #[props(default)]
    class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct CardTitleProps {
    /// The content to be displayed inside the card title
    children: Element,
    /// Additional CSS classes to apply to the title
    #[props(default)]
    class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct CardDescriptionProps {
    /// The content to be displayed inside the card description
    children: Element,
    /// Additional CSS classes to apply to the description
    #[props(default)]
    class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct CardContentProps {
    /// The content to be displayed inside the card content
    children: Element,
    /// Additional CSS classes to apply to the content
    #[props(default)]
    class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct CardFooterProps {
    /// The content to be displayed inside the card footer
    children: Element,
    /// Additional CSS classes to apply to the footer
    #[props(default)]
    class: Option<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct CardActionProps {
    /// The content to be displayed inside the card action
    children: Element,
    /// Additional CSS classes to apply to the action
    #[props(default)]
    class: Option<String>,
}

/// Card component
#[component]
pub fn Card(props: CardProps) -> Element {
    let mut class =
        vec!["bg-card text-card-foreground flex flex-col rounded-xl border".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card", class: class.join(" "), {props.children} }
    }
}

/// CardHeader component
#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let mut class = vec!["@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-1.5 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card-header", class: class.join(" "), {props.children} }
    }
}

/// CardTitle component
#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let mut class = vec!["leading-none font-semibold".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card-title", class: class.join(" "), {props.children} }
    }
}

/// CardDescription component
#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let mut class = vec!["text-muted-foreground text-sm".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card-description", class: class.join(" "), {props.children} }
    }
}

/// CardAction component
#[component]
pub fn CardAction(props: CardActionProps) -> Element {
    let mut class =
        vec!["col-start-2 row-span-2 row-start-1 self-start justify-self-end".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card-action", class: class.join(" "), {props.children} }
    }
}

/// CardContent component
#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let mut class = vec!["px-6".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card-content", class: class.join(" "), {props.children} }
    }
}

/// CardFooter component
#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    let mut class = vec!["flex items-center px-6 [.border-t]:pt-6".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "card-footer", class: class.join(" "), {props.children} }
    }
}
