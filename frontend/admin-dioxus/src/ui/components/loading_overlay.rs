use dioxus::prelude::*;
use std::time::Duration;

#[derive(Props, PartialEq, Clone)]
pub struct LoadingOverlayProps {
    #[props(default = false)]
    pub visible: ReadSignal<bool>,
}
const FADE_MS: u64 = 400;

#[component]
pub fn LoadingOverlay(props: LoadingOverlayProps) -> Element {
    let mut should_render = use_signal(|| false);
    let visible_sig = props.visible.clone();
    let visible_for_render = visible_sig.clone();

    use_effect(move || {
        let is_visible = visible_sig();
        if is_visible && !should_render() {
            should_render.set(true);
        } else if !is_visible && should_render() {
            spawn(async move {
                dioxus_time::sleep(Duration::from_millis(FADE_MS)).await;
                should_render.set(false);
            });
        }
    });

    if !should_render() {
        return rsx! { Fragment {} };
    }

    let opacity = if visible_for_render() {
        "opacity-100"
    } else {
        "opacity-0"
    };

    rsx! {
        div { class: format!("absolute inset-0 z-10 bg-background/50 backdrop-blur-[1px] flex items-center justify-center duration-400 ease-in-out {}", opacity),
            div { class: "h-6 w-6 rounded-full border-2 border-zinc-300 border-t-zinc-700 animate-spin" }
        }
    }
}
