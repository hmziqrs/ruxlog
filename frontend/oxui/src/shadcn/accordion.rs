use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::LdChevronDown;
use hmziq_dioxus_free_icons::Icon;

#[derive(PartialEq, Clone)]
pub struct AccordionContext {
    pub open_value: Option<i32>,
}

impl AccordionContext {
    pub fn new() -> Self {
        Self { open_value: None }
    }

    pub fn toggle(&mut self, value: i32) {
        if self.open_value == Some(value) {
            self.open_value = None;
        } else {
            self.open_value = Some(value);
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct AccordionProps {
    #[props(optional)]
    pub class: String,
    pub children: Element,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn Accordion(props: AccordionProps) -> Element {
    use_context_provider(|| Signal::new(AccordionContext::new()));

    rsx! {
        div {
            class: format!("data-slot-accordion {}", props.class),
            ..props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct AccordionItemProps {
    #[props(optional)]
    pub class: String,
    pub children: Element,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    pub value: i32,
}

#[derive(PartialEq, Clone)]
pub struct AccordionItemState(pub i32);

#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    use_context_provider(|| Signal::new(AccordionItemState(props.value)));

    rsx! {
        div {
            class: format!("data-slot-accordion-item border-b last:border-b-0 {}", props.class),
            ..props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct AccordionTriggerProps {
    #[props(optional)]
    pub class: String,
    pub children: Element,
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    let mut state_context = use_context::<Signal<AccordionContext>>();
    let item_context = use_context::<Signal<AccordionItemState>>();
    let item_value = item_context.read().0;
    let open_value = state_context.read().open_value;
    let is_open = open_value == Some(item_value);

    rsx! {
        div { class: "flex", ..props.attributes,
            button {
                class: format!(
                    "data-slot-accordion-trigger flex flex-1 items-start justify-between gap-4 rounded-md py-4 text-left text-sm font-medium transition-all outline-none hover:underline focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 {}",
                    props.class,
                ),
                onclick: move |evt| {
                    state_context.write().toggle(item_value);
                    if let Some(handler) = &props.onclick {
                        handler.call(evt);
                    }
                },
                {props.children}
                // Simple chevron icon replacement
                div {
                    class: format!(
                        "pointer-events-none w-4 h-4 shrink-0 translate-y-0.5 transition-transform duration-200 {}",
                        if is_open { "rotate-180" } else { "" },
                    ),
                    Icon { icon: LdChevronDown }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct AccordionContentProps {
    #[props(optional)]
    pub class: String,
    pub children: Element,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    let state_context = use_context::<Signal<AccordionContext>>();
    let item_context = use_context::<Signal<AccordionItemState>>();
    let item_value = item_context.read().0;
    let open_value = state_context.read().open_value;
    let is_open = open_value == Some(item_value);

    let display = if is_open { "block" } else { "hidden" };

    rsx! {
        div {
            class: "data-slot-accordion-content overflow-hidden text-sm {display}",
            ..props.attributes,
            div { class: "pt-0 pb-4", {props.children} }
        }
    }
}
