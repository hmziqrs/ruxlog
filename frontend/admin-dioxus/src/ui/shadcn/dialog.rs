use crate::ui::custom::AppPortal;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

/// Signal for managing dialog open/close state
#[derive(PartialEq)]
pub struct DialogContext(pub bool);

#[derive(Props, PartialEq, Clone)]
pub struct DialogProps {
    children: Element,
    #[props(default)]
    class: Option<String>,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    use_context_provider(|| Signal::new(DialogContext(false)));
    rsx! {
        div { "data-slot": "dialog", {props.children} }
    }
}

#[component]
pub fn DialogTrigger(props: DialogProps) -> Element {
    let mut dialog = use_context::<Signal<DialogContext>>();
    rsx! {
        button {
            "data-slot": "dialog-trigger",
            onclick: move |_| {
                let status = dialog.read().0;
                dialog.set(DialogContext(!status));
            },
            {props.children}
        }
    }
}

#[component]
pub fn DialogOverlay(props: DialogProps) -> Element {
    let mut class = vec!["data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dialog-overlay", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn DialogContent(props: DialogProps) -> Element {
    let mut dialog = use_context::<Signal<DialogContext>>();
    let mut class = vec!["bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg duration-200 sm:max-w-lg".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    if !dialog.read().0 {
        return rsx! {};
    }

    rsx! {
        AppPortal {
            DialogOverlay {}
            div {
                "data-slot": "dialog-content",
                "data-state": if dialog.read().0 { "open" } else { "closed" },
                class: class.join(" "),
                {props.children}
                button {
                    onclick: move |_| dialog.set(DialogContext(false)),
                    class: "ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none",
                    div { class: "w-4 h-4 pointer-events-none shrink-0",
                        Icon { icon: LdX {} }
                    }
                    span { class: "sr-only", "Close" }
                }
            }
        }
    }
}

#[component]
pub fn DialogHeader(props: DialogProps) -> Element {
    let mut class = vec!["flex flex-col gap-2 text-center sm:text-left".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dialog-header", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn DialogFooter(props: DialogProps) -> Element {
    let mut class = vec!["flex flex-col-reverse gap-2 sm:flex-row sm:justify-end".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dialog-footer", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn DialogTitle(props: DialogProps) -> Element {
    let mut class = vec!["text-lg leading-none font-semibold".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dialog-title", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn DialogDescription(props: DialogProps) -> Element {
    let mut class = vec!["text-muted-foreground text-sm".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "dialog-description", class: class.join(" "), {props.children} }
    }
}
