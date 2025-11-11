use crate::ui::{custom::AppPortal, shadcn::Button};
use dioxus::prelude::*;

/// Props for AlertDialog root component
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogProps {
    pub children: Element,

    #[props(default = None)]
    pub signal: Option<Signal<AlertDialogState>>,
}

pub struct AlertDialogState(bool);

/// Root provider component for AlertDialog
#[component]
pub fn AlertDialog(props: AlertDialogProps) -> Element {
    use_context_provider(|| {
        let signal = props
            .signal
            .unwrap_or_else(|| Signal::new(AlertDialogState(false)));
        signal
    });
    rsx! {
        {props.children}
    }
}

/// Props for AlertDialogTrigger
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogTriggerProps {
    pub children: Element,
}

/// Trigger component to open dialog
#[component]
pub fn AlertDialogTrigger(props: AlertDialogTriggerProps) -> Element {
    let mut open = use_context::<Signal<AlertDialogState>>();

    rsx! {
        button { onclick: move |_| open.set(AlertDialogState(false)), {props.children} }
    }
}

/// Props for AlertDialogPortal
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogPortalProps {
    pub children: Element,
}

/// Portal component for dialog content
#[component]
pub fn AlertDialogPortal(props: AlertDialogPortalProps) -> Element {
    let open: Signal<AlertDialogState> = use_context::<Signal<AlertDialogState>>();

    if open.read().0 {
        rsx! {
            AppPortal { {props.children} }
        }
    } else {
        rsx! {}
    }
}

/// Props for AlertDialogOverlay
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogOverlayProps {
    #[props(default)]
    pub class: Option<String>,
}

/// Overlay backdrop component
#[component]
pub fn AlertDialogOverlay(props: AlertDialogOverlayProps) -> Element {
    rsx! {
        div { class: props.class.clone().unwrap_or_else(|| "fixed inset-0 bg-black/50".to_string()) }
    }
}

/// Props for AlertDialogContent
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogContentProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

/// Content component for dialog body
#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    rsx! {
        div { class: props.class.clone().unwrap_or_else(|| "bg-white rounded-lg p-4".to_string()),
            {props.children}
        }
    }
}

/// Props for AlertDialogTitle
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogTitleProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

/// Title component
#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    rsx! {
        h2 { class: props.class.clone().unwrap_or_else(|| "text-lg font-bold".to_string()),
            {props.children}
        }
    }
}

/// Props for AlertDialogDescription
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogDescriptionProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

/// Description component
#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    rsx! {
        p { class: props.class.clone().unwrap_or_else(|| "text-sm text-gray-600".to_string()),
            {props.children}
        }
    }
}

/// Props for AlertDialogAction (confirm) button
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogActionProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
    #[props(optional)]
    onclick: Option<EventHandler<MouseEvent>>,
}

/// Action button closes dialog
#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let mut open = use_context::<Signal<AlertDialogState>>();
    rsx! {
        Button {
            class: props.class.clone().unwrap_or_else(|| "ml-auto".to_string()),
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
                open.set(AlertDialogState(false));
            },
            {props.children}
        }
    }
}

/// Props for AlertDialogCancel button
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogCancelProps {
    pub children: Element,
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let mut open = use_context::<Signal<AlertDialogState>>();
    rsx! {
        Button {
            class: props.class.clone().unwrap_or_else(|| "mr-2".to_string()),
            onclick: move |_| open.set(AlertDialogState(false)),
            {props.children}
        }
    }
}

// re-export for ease of use
pub use AlertDialog as Root;
pub use AlertDialogAction as Action;
pub use AlertDialogCancel as Cancel;
pub use AlertDialogContent as Content;
pub use AlertDialogDescription as Description;
pub use AlertDialogOverlay as Overlay;
pub use AlertDialogPortal as Portal;
pub use AlertDialogTitle as Title;
pub use AlertDialogTrigger as Trigger;
