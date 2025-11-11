use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct AvatarProps {
    /// Additional classes to apply to the avatar
    #[props(default)]
    class: Option<String>,
    children: Element,
}

#[derive(Props, PartialEq, Clone)]
pub struct AvatarImageProps {
    /// Additional classes to apply to the image
    #[props(default)]
    class: Option<String>,
    /// Image source URL
    src: String,
    /// Alt text for the image
    alt: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct AvatarFallbackProps {
    /// Additional classes to apply to the fallback
    #[props(default)]
    class: Option<String>,
    children: Element,
}

#[derive(PartialEq, Eq)]
pub enum AvatarImageStatus {
    Loading,
    Error,
    Success,
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    use_context_provider(|| Signal::new(AvatarImageStatus::Loading));
    let mut class = vec!["relative flex size-8 shrink-0 overflow-hidden rounded-full".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "avatar", class: class.join(" "), {props.children} }
    }
}

#[component]
pub fn AvatarImage(props: AvatarImageProps) -> Element {
    let mut class = vec!["aspect-square size-full".to_string()];
    let mut status = use_context::<Signal<AvatarImageStatus>>();

    let is_error = *status.read() == AvatarImageStatus::Error;
    let is_loading = *status.read() == AvatarImageStatus::Loading;

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    if is_error || props.src.is_empty() {
        class.push("hidden".to_string());
    } else {
        class.push("block".to_string());
    }

    rsx! {
        img {
            onload: move |_| {
                status.set(AvatarImageStatus::Success);
            },
            onerror: move |_| {
                status.set(AvatarImageStatus::Error);
            },
            "data-slot": "avatar-image",
            class: class.join(" "),
            src: props.src,
            alt: props.alt,
        }
        if is_loading {
            div { class: "loading loading-spinner loading-xs absolute" }
        }
    }
}

#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let mut class =
        vec!["flex size-full items-center justify-center rounded-full bg-muted".to_string()];

    if let Some(custom_class) = props.class {
        class.push(custom_class);
    }

    rsx! {
        div { "data-slot": "avatar-fallback", class: class.join(" "), {props.children} }
    }
}
