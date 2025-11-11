use dioxus::prelude::*;
use dioxus_time::sleep;
use std::time::Duration;

use crate::config::DarkMode;

#[component]
pub fn MouseTrackingCard(children: Element) -> Element {
    let mut mouse_pos = use_signal(|| (0.0, 0.0));
    // let mut card_rect: Signal<Rect<f64, Pixels>> = use_signal(|| Rect::default());
    let mut card_ref = use_signal(|| None as Option<std::rc::Rc<MountedData>>);
    let mut debounce_timer = use_signal(|| 0u64);

    let dark_mode = use_context::<Signal<DarkMode>>();
    let is_dark = dark_mode.read().0;

    rsx! {
        div {
            class: "relative w-full max-w-md",
            div {
                class: "absolute pointer-events-none transition-all duration-300 ease-out opacity-50",
                style: format!(
                    "left: {}px; top: {}px; transform: translate(-50%, -50%); width: 300px; height: 300px; border-radius: 50%; background: radial-gradient(circle, {} 0%, {} 70%); filter: blur(20px); z-index: 0;",
                    mouse_pos().0,
                    mouse_pos().1,
                    if is_dark { "rgba(244,244,245,0.3)" } else { "rgba(39,39,42,0.5)" },
                    if is_dark { "rgba(113,113,122,0)" } else { "rgba(212,212,216,0)" },
                ),
            }
            // Card with proper mouse tracking
            div {
                onmount: move |event| {
                    card_ref.set(Some(event.data()));
                },
                onmousemove: move |evt| {
                    let timer_id = debounce_timer() + 1;
                    debounce_timer.set(timer_id);

                    spawn(async move {
                        // Debounce: wait 100ms
                        sleep(Duration::from_millis(10)).await;

                        // Check if this is still the latest event
                        if debounce_timer() != timer_id {
                            return;
                        }

                        let rect = card_ref.peek();
                        if rect.is_none() {
                            info!("No rect available yet");
                            return;
                        }
                        let data = rect.as_ref().unwrap();
                        let rect = data.get_client_rect().await;
                        if rect.is_err() {
                            info!("No rect available");
                            return;
                        }
                        let rect = rect.unwrap();
                        let client = evt.client_coordinates();

                        let x = client.x - rect.origin.x;
                        let y = client.y - rect.origin.y;

                        // Clamp to card boundaries
                        let clamped_x = x.clamp(0.0, rect.size.width);
                        let clamped_y = y.clamp(0.0, rect.size.height);

                        mouse_pos.set((clamped_x, clamped_y));
                    });
                },
                class: "relative w-full overflow-visible rounded-2xl backdrop-blur-sm transition-colors duration-300",
                // Base border - always visible but subtle
                div {
                    class: "absolute inset-0 rounded-2xl pointer-events-none",
                    style: if is_dark { "background: transparent; box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);" } else { "background: transparent; box-shadow: inset 0 0 0 1px rgba(39, 39, 42, 0.08);" },
                }
                // Radial border highlight that follows mouse - much lighter now
                div {
                    class: "absolute inset-0 rounded-2xl pointer-events-none overflow-hidden",
                    style: format!(
                        "mask: radial-gradient(circle 100px at {}px {}px, white, transparent); -webkit-mask: radial-gradient(circle 100px at {}px {}px, white, transparent); box-shadow: inset 0 0 0 1px {}; transition: opacity 0.15s; opacity: {};",
                        mouse_pos().0,
                        mouse_pos().1,
                        mouse_pos().0,
                        mouse_pos().1,
                        if is_dark { "rgba(244,244,244,0.5)" } else { "rgba(39,39,42,0.4)" },
                        if mouse_pos().0 > 0.0 { "1" } else { "0" },
                    ),
                }
                // Card content
                div { class: "relative z-20 p-8 space-y-6",
                    {children}
                }
            }
        }
    }
}
