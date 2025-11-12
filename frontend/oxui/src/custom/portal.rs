use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AppPortalProps {
    #[props(default = String::new())]
    pub class: String,
    /// The content to be displayed inside the portal
    pub children: Element,
    #[props(default = "10".to_string())]
    pub z_index: String,
    #[props(default = None)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

static PORTAL_INDEX: std::sync::OnceLock<GlobalSignal<i32>> = std::sync::OnceLock::new();

pub fn use_index() -> &'static GlobalSignal<i32> {
    PORTAL_INDEX.get_or_init(|| GlobalSignal::new(|| 0))
}

#[component]
pub fn AppPortal(props: AppPortalProps) -> Element {
    // let portal_index=  use_index();
    // let mut z_index = use_signal(|| 10);

    // use_effect(move || {
    //     let index = portal_index() + 1;
    //     *portal_index.write() = index;
    //     z_index.set(z_index() + index);
    // });

    rsx! {
        div {
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            class: format!("fixed z-[{}] top-0 left-0 w-full h-full {}", props.z_index, props.class),
            {props.children}
        }
    }
}
