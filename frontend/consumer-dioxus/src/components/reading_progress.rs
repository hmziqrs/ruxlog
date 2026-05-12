use dioxus::prelude::*;

/// A reading progress bar that shows scroll position on post pages.
/// Renders a thin bar at the top of the viewport that fills as the user scrolls.
#[component]
pub fn ReadingProgressBar() -> Element {
    let mut progress = use_signal(|| 0u8);

    use_drop(move || {
        // Cleanup is handled by the effect ending
    });

    // Use a scroll event listener approach
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        let scroll_height = body.scroll_height() as f64;
                        let client_height = window.inner_height().unwrap_or(0.0.into()).as_f64().unwrap_or(0.0);
                        let scrollable = scroll_height - client_height;
                        if scrollable > 0.0 {
                            let scroll_top = window.scroll_y().unwrap_or(0.0);
                            let pct = ((scroll_top / scrollable) * 100.0) as u8;
                            progress.set(pct);
                        }
                    }
                }
            }
        });
    }

    let width = format!("{}%", progress());

    rsx! {
        div {
            class: "fixed top-0 left-0 z-50 h-1 bg-primary transition-all duration-150",
            style: "width: {width}",
        }
    }
}
