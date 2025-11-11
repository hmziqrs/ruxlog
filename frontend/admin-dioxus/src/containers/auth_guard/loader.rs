use dioxus::prelude::*;
use std::time::Duration;

#[derive(Props, PartialEq, Clone)]
pub struct AuthGuardLoaderProps {
    title: String,
    copy: String,
    #[props(default = false)]
    overlay: bool,
    pub show: ReadSignal<bool>,
}

#[component]
pub fn AuthGuardLoader(props: AuthGuardLoaderProps) -> Element {
    let container_class = if props.overlay {
        "fixed inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center px-4"
    } else {
        "min-h-screen bg-background flex items-center justify-center px-4"
    };

    const FADE_MS: u64 = 400;

    let mut should_render = use_signal(|| false);
    let show_sig = props.show.clone();
    let show_for_render = show_sig.clone();

    use_effect(move || {
        let is_show = show_sig();
        if is_show && !should_render() {
            should_render.set(true);
        } else if !is_show && should_render() {
            spawn(async move {
                dioxus_time::sleep(Duration::from_millis(FADE_MS)).await;
                should_render.set(false);
            });
        }
    });

    if !should_render() {
        return rsx! { Fragment {} };
    }

    let opacity = if show_for_render() {
        "opacity-100"
    } else {
        "opacity-0"
    };

    rsx! {
        div { class: "{container_class} duration-400 ease-in-out {opacity}",
            div { class: "w-full max-w-sm text-center space-y-6",
                div { class: "relative mx-auto h-24 w-24 flex items-center justify-center",
                    div { class: "absolute inset-0 rounded-full border-4 border-primary/20 border-t-primary animate-spin" }
                    img {
                        class: "h-12 w-12 relative",
                        src: asset!("/assets/logo.png"),
                        alt: "Ruxlog",
                    }
                }
                div { class: "space-y-2",
                    p { class: "text-lg font-semibold text-foreground", "{props.title}" }
                    p { class: "text-sm text-muted-foreground", "{props.copy}" }
                }
            }
        }
    }
}
