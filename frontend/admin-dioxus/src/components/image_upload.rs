use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdPlus, LdX},
    Icon,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, Url};

static INPUT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Props, PartialEq, Clone)]
pub struct ImageUploadProps {
    /// Current image URL (None or empty string means no image)
    #[props(default)]
    pub value: Option<String>,
    /// Called with the new URL when uploaded, or empty string when removed
    pub onchange: EventHandler<String>,
    /// Primary call-to-action label
    pub title: String,
    /// Helper text shown in the empty state
    pub description: String,
    /// Tailwind aspect ratio utility (e.g. "aspect-video")
    #[props(default = "aspect-video".to_string())]
    pub aspect_ratio: String,
}

/// Simple image picker that creates a blob URL for the selected file
#[component]
pub fn ImageUpload(props: ImageUploadProps) -> Element {
    let mut last_blob_url = use_signal(|| Option::<String>::None);
    let input_id = use_signal(|| {
        format!(
            "image-upload-{}",
            INPUT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    });

    let value = props.value.clone().unwrap_or_default();
    let has_image = !value.trim().is_empty();

    // Open native file picker
    let open_picker = {
        let id = input_id();
        move |_| {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(el) = doc.get_element_by_id(&id) {
                        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                            let _ = input.click();
                        }
                    }
                }
            }
        }
    };

    // Handle file selection -> create object URL and emit
    let on_file_change = {
        let id = input_id();
        move |_| {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(el) = doc.get_element_by_id(&id) {
                        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                            if let Some(files) = input.files() {
                                if let Some(file) = files.get(0) {
                                    if let Some(prev) = last_blob_url() {
                                        let _ = Url::revoke_object_url(&prev);
                                    }
                                    // File extends Blob in the web APIs; cast for Rust types
                                    let js_val: &wasm_bindgen::JsValue = file.as_ref();
                                    let blob: &web_sys::Blob = js_val.unchecked_ref();
                                    if let Ok(url) = Url::create_object_url_with_blob(blob) {
                                        last_blob_url.set(Some(url.clone()));
                                        props.onchange.call(url);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    // Remove image
    let handle_remove = {
        let id = input_id();
        move |_| {
            if let Some(prev) = last_blob_url() {
                let _ = Url::revoke_object_url(&prev);
                last_blob_url.set(None);
            }
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(el) = doc.get_element_by_id(&id) {
                        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                            input.set_value("");
                        }
                    }
                }
            }
            props.onchange.call(String::new());
        }
    };

    rsx! {
        div { class: "space-y-2",
            // Hidden file input for picking images
            input {
                r#type: "file",
                accept: "image/*",
                class: "hidden",
                id: "{input_id()}",
                onchange: on_file_change,
            }
            if has_image {
                div { class: "relative overflow-hidden rounded-md border border-zinc-200 dark:border-zinc-800",
                    img { src: value, alt: props.title.clone(), class: format!("w-full h-auto object-cover {}", props.aspect_ratio) }
                    Button {
                        r#type: "button".to_string(),
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Icon,
                        onclick: handle_remove,
                        class: "absolute top-2 right-2 h-8 w-8 rounded-full".to_string(),
                        Icon { icon: LdX }
                        span { class: "sr-only", "Remove image" }
                    }
                }
            } else {
                div { class: "border border-dashed rounded-md p-8 text-center border-zinc-200 dark:border-zinc-800",
                    div { class: "flex flex-col items-center gap-2",
                        div { class: "p-3 rounded-full bg-zinc-100 dark:bg-zinc-800",
                            Icon { icon: LdPlus, class: "h-6 w-6 text-zinc-500 dark:text-zinc-400" }
                        }
                        div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                            p { "{props.description}" }
                        }
                        Button {
                            r#type: "button".to_string(),
                            variant: ButtonVariant::Outline,
                            onclick: open_picker,
                            class: "mt-2 border-zinc-200 dark:border-zinc-800".to_string(),
                            "{props.title}"
                        }
                    }
                }
            }
        }
    }
}
