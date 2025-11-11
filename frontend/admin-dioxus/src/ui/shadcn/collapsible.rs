use dioxus::prelude::*;

/// Signal for managing collapsible open/close state
#[derive(PartialEq)]
pub struct CollapsibleContext(pub bool);

#[derive(Props, PartialEq, Clone)]
pub struct CollapsibleProps {
    children: Element,
    #[props(default)]
    class: Option<String>,
}

#[component]
pub fn Collapsible(props: CollapsibleProps) -> Element {
    use_context_provider(|| Signal::new(CollapsibleContext(false)));

    let mut class = vec![];
    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "collapsible", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn CollapsibleTrigger(props: CollapsibleProps) -> Element {
    let mut open = use_context::<Signal<CollapsibleContext>>();

    let mut class = vec![];
    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        button {
            "data-slot": "collapsible-trigger",
            "data-state": if open.read().0 { "open" } else { "closed" },
            class: class.join(" "),
            onclick: move |_| {
                let status = open.read().0;
                open.set(CollapsibleContext(!status));
            },
            {props.children}
        }
    }
}

#[component]
pub fn CollapsibleContent(props: CollapsibleProps) -> Element {
    let open = use_context::<Signal<CollapsibleContext>>();

    let mut class = vec![];
    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    if !open.read().0 {
        return rsx! {};
    }

    rsx! {
        div {
            "data-slot": "collapsible-content",
            "data-state": if open.read().0 { "open" } else { "closed" },
            class: class.join(" "),
            {props.children}
        }
    }
}
