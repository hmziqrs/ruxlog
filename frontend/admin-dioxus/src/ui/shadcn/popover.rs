use dioxus::{logger::tracing, prelude::*};
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

use crate::ui::custom::AppPortal;

/// Signal for managing popover open/close state
#[derive(PartialEq)]
pub struct PopoverContext(pub bool);

/// Properties for all popover components
#[derive(Props, PartialEq, Clone)]
pub struct PopoverProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// Root Popover component
#[component]
pub fn Popover(props: PopoverProps) -> Element {
    use_context_provider(|| Signal::new(PopoverContext(false)));

    rsx! {
        div { "data-slot": "popover", {props.children} }
    }
}

/// PopoverTrigger component
#[component]
pub fn PopoverTrigger(props: PopoverProps) -> Element {
    let mut open = use_context::<Signal<PopoverContext>>();
    rsx! {
        button {
            "data-slot": "popover-trigger",
            class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50",
            onclick: move |e| {
                let rect = e.page_coordinates();
                tracing::info!("Popover button clicked at: {:?}", rect);
                let is_open = open.peek().0;
                open.set(PopoverContext(!is_open));
            },
            {props.children}
        }
    }
}

/// PopoverAnchor component for custom anchor positioning
#[derive(Props, PartialEq, Clone)]
pub struct PopoverAnchorProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// PopoverAnchor component
#[component]
pub fn PopoverAnchor(props: PopoverAnchorProps) -> Element {
    let class_str = props.class.clone().unwrap_or_default();

    rsx! {
        div { "data-slot": "popover-anchor", class: class_str, {props.children} }
    }
}

/// PopoverContent component
#[component]
pub fn PopoverContent(props: PopoverProps) -> Element {
    let mut open = use_context::<Signal<PopoverContext>>();
    let is_open = open.read().0;

    let mut class = vec![
        "absolute bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-72 origin-[var(--radix-popover-content-transform-origin)] rounded-md border p-4 shadow-md outline-hidden".to_string()
    ];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        AppPortal {
            onclick: move |_| {
                open.set(PopoverContext(false));
            },
            class: format!("{}", if is_open { "block" } else { "hidden" }).to_string(),
        }
        div {
            onclick: move |e| {
                e.stop_propagation();
            },
            "data-slot": "popover-content",
            "data-state": if is_open { "open" } else { "closed" },
            class: class.join(" "),
            style: format!("display: {}", if is_open { "block" } else { "none" }),
            {props.children}
            button {
                r#type: "button",
                onclick: move |_| open.set(PopoverContext(false)),
                class: "absolute right-2 top-2 inline-flex items-center justify-center rounded-lg p-1 text-popover-foreground/50 opacity-70 hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
                div { class: "h-4 w-4",
                    Icon { icon: LdX {} }
                }
                span { class: "sr-only", "Close" }
            }
        }
    }
}

/// PopoverClose component for custom close buttons
#[derive(Props, PartialEq, Clone)]
pub struct PopoverCloseProps {
    /// Child elements to render inside the component
    pub children: Element,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// PopoverClose component
#[component]
pub fn PopoverClose(props: PopoverCloseProps) -> Element {
    let mut open = use_context::<Signal<PopoverContext>>();
    let class_str = props.class.clone().unwrap_or_default();

    rsx! {
        button {
            "data-slot": "popover-close",
            class: class_str,
            onclick: move |_| open.set(PopoverContext(false)),
            {props.children}
        }
    }
}
