use crate::utils::persist;
use dioxus::{logger::tracing, prelude::*};
use oxui::components::SonnerToaster;

pub mod components;
mod config;
pub mod containers;
pub mod env;
pub mod hooks;
pub mod router;
pub mod screens;
pub mod utils;

#[allow(unused_imports)]
use utils::js_bridge;

fn main() {
    // Configure HTTP client base URL only. The per-session CSRF token is no
    // longer baked in at build time (plan Phase 5); it is fetched from
    // `/csrf/v1/generate` on boot (see App) and after login.
    println!("APP_API_URL: {}", env::APP_API_URL);

    // Ensure URL has protocol
    let base_url = if env::APP_API_URL.starts_with("http") {
        env::APP_API_URL.to_string()
    } else {
        format!("http://{}", env::APP_API_URL)
    };

    oxcore::http::configure(base_url);

    dioxus::launch(App);
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    // let toast = use_context_provider(|| Signal::new(ToastManager::default()));
    tracing::info!("APP_API_URL: {}", env::APP_API_URL);
    // Initialize document theme from persistent storage on app mount.
    use_effect(|| {
        let stored = persist::get_theme();
        spawn(async move {
            match stored.as_deref() {
                Some("dark") => {
                    let _ = document::eval("document.documentElement.classList.add('dark');").await;
                }
                Some("light") => {
                    let _ =
                        document::eval("document.documentElement.classList.remove('dark');").await;
                }
                _ => {}
            }
        });
    });

    // Fetch the per-session CSRF token on boot. The backend bootstraps/
    // rehydrates the session and returns the bound token, attached to every
    // mutating request thereafter. Login re-fetches it too.
    use_effect(|| {
        spawn(async move {
            if let Err(e) = oxcore::http::refresh_csrf_token().await {
                tracing::warn!("Failed to refresh CSRF token on boot: {e}");
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            "crossorigin": "",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Geist+Mono:wght@400..600&family=Geist:wght@400..600&display=swap",
        }
        // document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        SonnerToaster { Router::<crate::router::Route> {} }
    }
}
// ToastFrame component is temporarily commented out due to compatibility issues
// dioxus_toast::ToastFrame { manager: toast, style: None }
