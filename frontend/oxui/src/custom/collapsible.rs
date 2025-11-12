#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

// Helper function to get state string
fn get_state(open: bool) -> &'static str {
    if open {
        "open"
    } else {
        "closed"
    }
}

// --- Context ---

#[derive(Clone, PartialEq)]
struct CollapsibleContext {
    content_id: Signal<String>,
    is_disabled: Signal<bool>,
    is_open: Signal<bool>,
    on_open_toggle: Rc<RefCell<Option<EventHandler<()>>>>,
}

impl CollapsibleContext {
    fn new(
        content_id: Signal<String>,
        is_disabled: Signal<bool>,
        is_open: Signal<bool>,
        on_open_toggle: Rc<RefCell<Option<EventHandler<()>>>>,
    ) -> Self {
        Self {
            content_id,
            is_disabled,
            is_open,
            on_open_toggle,
        }
    }

    fn toggle(&self) {
        if let Some(handler) = self.on_open_toggle.borrow().as_ref() {
            handler.call(());
        }
    }
}

fn use_collapsible_context() -> CollapsibleContext {
    use_context::<CollapsibleContext>()
}

// --- Collapsible (Root) ---

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleProps {
    #[props(default = false)]
    default_open: bool,
    #[props(optional)]
    open: Option<bool>,
    #[props(default = false)]
    disabled: bool,
    #[props(optional)]
    on_open_change: Option<EventHandler<bool>>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
}

#[component]
pub fn AppCollapsible(props: CollapsibleProps) -> Element {
    let is_controlled = props.open.is_some();
    let initial_open = if is_controlled {
        props.open.unwrap()
    } else {
        props.default_open
    };
    let mut is_open = use_signal(|| initial_open);
    let mut is_disabled = use_signal(|| props.disabled);
    let content_id = use_signal(|| format!("collapsible-content-{}", Uuid::new_v4()));

    // Use Rc<RefCell<Option<EventHandler>>> to store the callback
    let on_open_change_callback = props.on_open_change.clone();
    let on_open_change_ref = use_signal(|| Rc::new(RefCell::new(on_open_change_callback)));

    // Update internal state if controlled prop changes
    use_effect(move || {
        if let Some(controlled_open) = props.open {
            if controlled_open != *is_open.read() {
                is_open.set(controlled_open);
            }
        }
    });

    // Update disabled state if prop changes
    use_effect(move || {
        if props.disabled != *is_disabled.read() {
            is_disabled.set(props.disabled);
        }
    });

    let handle_open_toggle = Rc::new(RefCell::new(Some(EventHandler::new(move |_| {
        if !*is_disabled.read() {
            let new_open_state = !*is_open.read();
            if !is_controlled {
                is_open.set(new_open_state);
            }
            if let Some(callback) = on_open_change_ref.read().borrow().as_ref() {
                callback.call(new_open_state);
            }
        }
    }))));

    use_context_provider(|| {
        CollapsibleContext::new(content_id, is_disabled, is_open, handle_open_toggle.clone())
    });

    let current_open_state = *is_open.read();
    let current_disabled_state = *is_disabled.read();

    rsx! {
        div {
            "data-state": get_state(current_open_state),
            "data-disabled": if current_disabled_state { Some("") } else { None },
            ..props.attributes,
            {props.children}
        }
    }
}

// --- CollapsibleTrigger ---

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleTriggerProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
}

#[component]
pub fn AppCollapsibleTrigger(props: CollapsibleTriggerProps) -> Element {
    let context = use_collapsible_context();
    let is_open = *context.is_open.read();
    let is_disabled = *context.is_disabled.read();
    let content_id = context.content_id.read().clone();

    rsx! {
        button {
            "type": "button",
            "aria-controls": content_id,
            "aria-expanded": is_open.to_string(),
            "data-state": get_state(is_open),
            "data-disabled": if is_disabled { Some("") } else { None },
            disabled: is_disabled,
            onclick: move |_| {
                context.toggle();
            },
            ..props.attributes,
            {props.children}
        }
    }
}

// --- CollapsibleContent ---

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleContentProps {
    #[props(default = false)]
    force_mount: bool,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
}

#[component]
pub fn AppCollapsibleContent(props: CollapsibleContentProps) -> Element {
    let context = use_collapsible_context();
    let is_open = *context.is_open.read();
    let is_disabled = *context.is_disabled.read();
    let content_id = context.content_id.read().clone();

    // Basic visibility control. Animation requires more complex handling (JS interop or CSS).
    let is_visible = props.force_mount || is_open;

    if !is_visible {
        return rsx! {};
    }

    rsx! {
        div {
            "data-state": get_state(is_open),
            "data-disabled": if is_disabled { Some("") } else { None },
            id: content_id,
            // Use `hidden` attribute for simple show/hide, or rely on CSS transitions/animations based on `data-state`
            hidden: if is_visible { None } else { Some(true) },
            ..props.attributes,
            {props.children}
        }
    }
}

// // --- Exports ---
// pub const Root: fn(CollapsibleProps) -> Element = Collapsible;
// pub const Trigger: fn(CollapsibleTriggerProps) -> Element = CollapsibleTrigger;
// pub const Content: fn(CollapsibleContentProps) -> Element = CollapsibleContent;
