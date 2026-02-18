use dioxus::{logger::tracing, prelude::*};

use crate::components::AmbientCanvasBackground;
use oxui::components::SonnerToaster;

pub mod components;
mod config;
pub mod containers;
pub mod env;
pub mod hooks;
pub mod router;
pub mod screens;
pub mod seo;
#[cfg(feature = "server")]
pub mod server;
pub mod server_fns;
pub mod utils;

#[cfg(feature = "analytics")]
pub mod analytics;

fn configure_http_client() {
    use base64::prelude::*;

    // Configure HTTP client
    println!("APP_API_URL: {}", env::APP_API_URL);
    println!("APP_CSRF_TOKEN: {}", env::APP_CSRF_TOKEN);

    let base_url = if env::APP_API_URL.starts_with("http") {
        env::APP_API_URL.to_string()
    } else {
        format!("http://{}", env::APP_API_URL)
    };
    let csrf_token = BASE64_STANDARD.encode(env::APP_CSRF_TOKEN.as_bytes());
    oxcore::http::configure(base_url, csrf_token);
}

#[cfg(feature = "server")]
fn main() {
    configure_http_client();

    dioxus::LaunchBuilder::new()
        .with_cfg(server_only! {
            dioxus::server::ServeConfig::default()
        })
        .launch(App);
}

#[cfg(all(
    feature = "web",
    not(any(feature = "server", feature = "desktop", feature = "mobile"))
))]
fn main() {
    configure_http_client();

    dioxus::LaunchBuilder::new()
        .with_cfg(web! {
            dioxus::web::Config::default()
        })
        .launch(App);
}

// Desktop, mobile, or any other non-server/non-web build (dx builds native
// clients without enabling the desktop/mobile Cargo features)
#[cfg(not(any(feature = "server", feature = "web")))]
fn main() {
    configure_http_client();
    dioxus::launch(App);
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    tracing::info!("APP_API_URL: {}", env::APP_API_URL);
    tracing::info!("APP_CSRF_TOKEN: {}", env::APP_CSRF_TOKEN);

    // Initialize Firebase Analytics (WASM-only)
    #[cfg(all(target_arch = "wasm32", feature = "analytics"))]
    use_effect(|| {
        if analytics::initialize() {
            tracing::info!("Firebase Analytics enabled");
        } else {
            tracing::warn!("Firebase Analytics initialization failed - check configuration");
        }
    });

    // Initialize document theme from persistent storage on app mount (WASM-only)
    // Defaults to dark mode when no preference is stored
    #[cfg(target_arch = "wasm32")]
    use_effect(|| {
        let stored = utils::persist::get_theme();
        spawn(async move {
            match stored.as_deref() {
                Some("light") => {
                    let _ =
                        document::eval("document.documentElement.classList.remove('dark');").await;
                }
                _ => {
                    // Default to dark mode for "dark", None, or any other value
                    let _ = document::eval("document.documentElement.classList.add('dark');").await;
                }
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
        AmbientCanvasBackground {}
        div { style: "position: relative; z-index: 10;",
            SuspenseBoundary {
                fallback: |_| rsx! {
                    div { class: "min-h-screen flex items-center justify-center",
                        div { class: "animate-pulse text-muted-foreground", "Loading..." }
                    }
                },
                SonnerToaster { Router::<crate::router::Route> {} }
            }
        }
    }
}
