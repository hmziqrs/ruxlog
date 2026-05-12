use dioxus::prelude::*;

/// GDPR-compliant cookie consent banner.
/// Shows on first visit, stores preference in localStorage.
#[component]
pub fn CookieConsent() -> Element {
    let mut visible = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            let stored = js_sys::Reflect::get(&web_sys::window().unwrap().local_storage().unwrap().unwrap(), &wasm_bindgen::JsValue::from_str("cookie_consent"))
                .ok();
            match stored {
                Some(val) => !js_sys::Boolean::from(val).value_of(),
                None => true,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        { false }
    });

    let on_accept = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("cookie_consent", "true");
                }
            }
        }
        visible.set(false);
    };

    let on_decline = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("cookie_consent", "false");
                }
            }
        }
        visible.set(false);
    };

    if !visible() {
        return rsx! {};
    }

    rsx! {
        div { class: "fixed bottom-0 left-0 right-0 z-50 p-4 bg-background/95 backdrop-blur-sm border-t border-border",
            div { class: "container mx-auto max-w-4xl flex flex-col sm:flex-row items-center justify-between gap-4",
                p { class: "text-sm text-muted-foreground",
                    "We use cookies to analyze traffic and improve your experience. No third-party tracking."
                }
                div { class: "flex gap-2 shrink-0",
                    button {
                        class: "px-4 py-2 text-sm rounded-lg border border-border hover:bg-muted/50 transition-colors",
                        onclick: on_decline,
                        "Decline"
                    }
                    button {
                        class: "px-4 py-2 text-sm rounded-lg bg-primary text-primary-foreground hover:opacity-90 transition-opacity",
                        onclick: on_accept,
                        "Accept"
                    }
                }
            }
        }
    }
}
