use dioxus::{logger::tracing, prelude::*};

use crate::utils::persist;
use oxui::components::SonnerToaster;

pub mod components;
mod config;
pub mod containers;
pub mod env;
pub mod hooks;
pub mod router;
pub mod screens;
pub mod seo;
pub mod utils;

#[cfg(feature = "analytics")]
pub mod analytics;

#[allow(unused_imports)]
// use utils::js_bridge;

fn main() {
    use base64::prelude::*;

    // Configure HTTP client - oxcore handles platform differences (gloo-net for WASM, reqwest for native)
    println!("APP_API_URL: {}", env::APP_API_URL);
    println!("APP_CSRF_TOKEN: {}", env::APP_CSRF_TOKEN);

    // Ensure URL has protocol
    let base_url = if env::APP_API_URL.starts_with("http") {
        env::APP_API_URL.to_string()
    } else {
        format!("http://{}", env::APP_API_URL)
    };

    let csrf_token = BASE64_STANDARD.encode((env::APP_CSRF_TOKEN).as_bytes());
    oxcore::http::configure(base_url, csrf_token);

    dioxus::launch(App);
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    tracing::info!("APP_API_URL: {}", env::APP_API_URL);
    tracing::info!("APP_CSRF_TOKEN: {}", env::APP_CSRF_TOKEN);

    // Initialize Firebase Analytics
    #[cfg(all(target_arch = "wasm32", feature = "analytics"))]
    use_effect(|| {
        if analytics::initialize() {
            tracing::info!("Firebase Analytics enabled");
        } else {
            tracing::warn!("Firebase Analytics initialization failed - check configuration");
        }
    });

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
        SonnerToaster { Router::<crate::router::Route> {} }
    }
}
