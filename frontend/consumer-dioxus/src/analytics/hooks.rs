//! Dioxus hooks for analytics tracking
//!
//! This module provides React-like hooks for automatic tracking
//! of user behavior in Dioxus components.

use dioxus::prelude::*;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, Element};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use super::tracker;

/// Hook to track time spent on a page
///
/// Automatically tracks how long a user stays on a page.
/// Fires the event when the component unmounts.
///
/// # Arguments
/// * `route` - The current route/page path
///
/// # Example
/// ```rust
/// use_page_timer("/blog/my-post");
/// ```
#[cfg(target_arch = "wasm32")]
pub fn use_page_timer(route: &str) {
    let route = route.to_string();
    let start_time = use_signal(|| js_sys::Date::now());

    use_drop(move || {
        let duration = (js_sys::Date::now() - start_time()) / 1000.0;
        tracker::track_time_on_page(&route, duration);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn use_page_timer(_route: &str) {
    // No-op on non-wasm targets
}

/// Hook to track scroll depth milestones
///
/// Tracks when user scrolls to 25%, 50%, 75%, and 100% of the page.
/// Prevents duplicate tracking of the same milestone.
///
/// # Arguments
/// * `route` - The current route/page path
///
/// # Example
/// ```rust
/// use_scroll_depth("/blog/my-post");
/// ```
#[cfg(target_arch = "wasm32")]
pub fn use_scroll_depth(route: &str) {
    use std::rc::Rc;
    use std::cell::Cell;

    let route = route.to_string();
    let mut milestones = use_signal(|| vec![false; 4]); // [25%, 50%, 75%, 100%]

    use_effect(move || {
        if let Some(window) = window() {

        let route_clone = route.clone();
        let mut milestones_clone = milestones.clone();

        let closure = Rc::new(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };

            let document = match window.document() {
                Some(d) => d,
                None => return,
            };

            // Get scroll position and document height
            let scroll_top = window.scroll_y().unwrap_or(0.0);
            let window_height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);
            let document_height = if let Some(body) = document.body() {
                body.scroll_height() as f64
            } else if let Some(element) = document.document_element() {
                element.scroll_height() as f64
            } else {
                0.0
            };

            if document_height == 0.0 {
                return;
            }

            // Calculate scroll percentage
            let scroll_percentage = ((scroll_top + window_height) / document_height * 100.0) as u8;

            // Check milestones
            let mut milestones_state = milestones_clone.read().clone();
            let route = route_clone.clone();

            if scroll_percentage >= 25 && !milestones_state[0] {
                milestones_state[0] = true;
                tracker::track_scroll_depth(&route, 25);
            }
            if scroll_percentage >= 50 && !milestones_state[1] {
                milestones_state[1] = true;
                tracker::track_scroll_depth(&route, 50);
            }
            if scroll_percentage >= 75 && !milestones_state[2] {
                milestones_state[2] = true;
                tracker::track_scroll_depth(&route, 75);
            }
            if scroll_percentage >= 100 && !milestones_state[3] {
                milestones_state[3] = true;
                tracker::track_scroll_depth(&route, 100);
            }

            milestones_clone.set(milestones_state);
        }) as Box<dyn FnMut()>));

        // Use throttling to prevent excessive event firing
        let last_call = Rc::new(Cell::new(0.0));
        let throttle_ms = 500.0;

        let throttled_closure_inner = closure.clone();
        let last_call_clone = last_call.clone();

        let throttled_closure = Rc::new(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            let now = js_sys::Date::now();
            if now - last_call_clone.get() >= throttle_ms {
                last_call_clone.set(now);
                let _ = throttled_closure_inner
                    .as_ref()
                    .as_ref()
                    .unchecked_ref::<js_sys::Function>()
                    .call0(&wasm_bindgen::JsValue::NULL);
            }
        }) as Box<dyn FnMut()>));

        // Add scroll event listener
        let _ = window.add_event_listener_with_callback(
            "scroll",
            throttled_closure.as_ref().as_ref().unchecked_ref()
        );

        // Initial check
        let _ = closure
            .as_ref()
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .call0(&wasm_bindgen::JsValue::NULL);

        // Cleanup function
        use_drop(move || {
            let _ = window.remove_event_listener_with_callback(
                "scroll",
                throttled_closure.as_ref().as_ref().unchecked_ref(),
            );
        });
        }
    });
}

/// No-op version for non-wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub fn use_scroll_depth(_route: &str) {
    // No-op on non-wasm targets
}

/// Hook to track outbound link clicks in content
///
/// Adds event delegation to track clicks on external links within a container.
///
/// # Arguments
/// * `container_id` - The ID of the container element to monitor
/// * `post_id` - Optional post ID for context
///
/// # Example
/// ```rust
/// use_outbound_link_tracker("post-content", Some("post-123"));
/// ```
#[cfg(target_arch = "wasm32")]
pub fn use_outbound_link_tracker(container_id: &str, post_id: Option<String>) {
    use std::rc::Rc;

    let container_id = container_id.to_string();

    use_effect(move || {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(container) = document.get_element_by_id(&container_id) {

        let current_origin = window.location().origin().unwrap_or_default();
        let post_id_clone = post_id.clone();

        let closure = Rc::new(wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
            if let Some(target) = event.target() {
                if let Ok(element) = target.dyn_into::<Element>() {
                    if element.tag_name() == "A" {
                        if let Some(href) = element.get_attribute("href") {
                            // Check if it's an external link
                            if href.starts_with("http") && !href.starts_with(&current_origin) {
                                tracker::track_outbound_link(&href, post_id_clone.as_deref());
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>));

        let _ = container.add_event_listener_with_callback(
            "click",
            closure.as_ref().as_ref().unchecked_ref()
        );

        // Cleanup
        use_drop(move || {
            let _ = container.remove_event_listener_with_callback(
                "click",
                closure.as_ref().as_ref().unchecked_ref(),
            );
        });
                }
            }
        }
    });
}

/// No-op version for non-wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub fn use_outbound_link_tracker(_container_id: &str, _post_id: Option<String>) {
    // No-op on non-wasm targets
}
