//! Dioxus hooks for analytics tracking
//!
//! This module provides React-like hooks for automatic tracking
//! of user behavior in Dioxus components.

use dioxus::prelude::*;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, Element};

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
pub fn use_page_timer(route: &str) {
    let route = route.to_string();

    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let start_time = js_sys::Date::now();
            let route_clone = route.clone();

            // Return cleanup function
            move || {
                let duration = (js_sys::Date::now() - start_time) / 1000.0;
                tracker::track_time_on_page(&route_clone, duration);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = route;
            move || {}
        }
    });
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
    let route = route.to_string();
    let mut milestones = use_signal(|| vec![false; 4]); // [25%, 50%, 75%, 100%]

    use_effect(move || {
        let window = match window() {
            Some(w) => w,
            None => return move || {},
        };

        let document = match window.document() {
            Some(d) => d,
            None => return move || {},
        };

        let route_clone = route.clone();
        let milestones_clone = milestones.clone();

        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
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
        }) as Box<dyn FnMut()>);

        // Use throttling to prevent excessive event firing
        let mut last_call = 0.0;
        let throttle_ms = 500.0;

        let throttled_closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            let now = js_sys::Date::now();
            if now - last_call >= throttle_ms {
                last_call = now;
                closure.as_ref().unchecked_ref::<js_sys::Function>().call0(&wasm_bindgen::JsValue::NULL).ok();
            }
        }) as Box<dyn FnMut()>);

        // Add scroll event listener
        window
            .add_event_listener_with_callback("scroll", throttled_closure.as_ref().unchecked_ref())
            .ok();

        // Initial check
        closure.as_ref().unchecked_ref::<js_sys::Function>().call0(&wasm_bindgen::JsValue::NULL).ok();

        // Cleanup function
        let cleanup_window = window.clone();
        let cleanup_closure = throttled_closure.clone();
        move || {
            cleanup_window
                .remove_event_listener_with_callback("scroll", cleanup_closure.as_ref().unchecked_ref())
                .ok();
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
    let container_id = container_id.to_string();

    use_effect(move || {
        let window = match window() {
            Some(w) => w,
            None => return move || {},
        };

        let document = match window.document() {
            Some(d) => d,
            None => return move || {},
        };

        let container = match document.get_element_by_id(&container_id) {
            Some(c) => c,
            None => return move || {},
        };

        let current_origin = window.location().origin().unwrap_or_default();
        let post_id_clone = post_id.clone();

        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
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
        }) as Box<dyn FnMut(_)>);

        container
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .ok();

        // Cleanup
        let cleanup_container = container.clone();
        let cleanup_closure = closure.clone();
        move || {
            cleanup_container
                .remove_event_listener_with_callback("click", cleanup_closure.as_ref().unchecked_ref())
                .ok();
        }
    });
}

/// No-op version for non-wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub fn use_outbound_link_tracker(_container_id: &str, _post_id: Option<String>) {
    // No-op on non-wasm targets
}
