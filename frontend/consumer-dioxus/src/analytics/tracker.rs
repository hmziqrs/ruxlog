//! High-level event tracking functions
//!
//! This module provides user-friendly functions for tracking various events
//! in the application using Firebase Analytics.

use super::bindings;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Track a page view event
pub fn track_page_view(route: &str, title: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            bindings::log_page_view(route, title);
            log::debug!("Analytics: Page view - {} ({})", title, route);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (route, title);
    }
}

/// Track a post view event
pub fn track_post_view(post_id: &str, post_title: &str, category: Option<&str>) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let mut params = json!({
                "post_id": post_id,
                "post_title": post_title,
            });

            if let Some(cat) = category {
                params["category"] = json!(cat);
            }

            bindings::log_event("post_view", serde_wasm_bindgen::to_value(&params).unwrap());
            log::debug!("Analytics: Post view - {} ({})", post_title, post_id);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (post_id, post_title, category);
    }
}

/// Track time spent on a page
pub fn track_time_on_page(route: &str, duration_seconds: f64) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "page": route,
                "duration_seconds": duration_seconds,
                "duration_minutes": (duration_seconds / 60.0).round(),
            });

            bindings::log_event(
                "time_on_page",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!(
                "Analytics: Time on page - {} ({:.1}s)",
                route,
                duration_seconds
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (route, duration_seconds);
    }
}

/// Track scroll depth milestone
pub fn track_scroll_depth(route: &str, depth_percentage: u8) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "page": route,
                "depth_percentage": depth_percentage,
            });

            bindings::log_event(
                "scroll_depth",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!(
                "Analytics: Scroll depth - {} ({}%)",
                route,
                depth_percentage
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (route, depth_percentage);
    }
}

/// Track a like event
pub fn track_like(post_id: &str, post_title: &str, liked: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "post_id": post_id,
                "post_title": post_title,
                "action": if liked { "liked" } else { "unliked" },
            });

            bindings::log_event(
                "engagement_like",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!(
                "Analytics: Like - {} ({})",
                if liked { "liked" } else { "unliked" },
                post_id
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (post_id, post_title, liked);
    }
}

/// Track a share event
pub fn track_share(post_id: &str, post_title: &str, platform: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "post_id": post_id,
                "post_title": post_title,
                "platform": platform,
            });

            bindings::log_event(
                "engagement_share",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!("Analytics: Share - {} on {}", post_id, platform);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (post_id, post_title, platform);
    }
}

/// Track a comment event
pub fn track_comment(post_id: &str, post_title: &str, action: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "post_id": post_id,
                "post_title": post_title,
                "action": action,
            });

            bindings::log_event(
                "engagement_comment",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!("Analytics: Comment {} - {}", action, post_id);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (post_id, post_title, action);
    }
}

/// Track a category click event
pub fn track_category_click(category: &str, source: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "category": category,
                "source": source,
            });

            bindings::log_event(
                "navigation_category",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!("Analytics: Category click - {} from {}", category, source);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (category, source);
    }
}

/// Track a tag click event
pub fn track_tag_click(tag: &str, source: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "tag": tag,
                "source": source,
            });

            bindings::log_event(
                "navigation_tag",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!("Analytics: Tag click - {} from {}", tag, source);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (tag, source);
    }
}

/// Track a navigation event (navbar, footer, etc.)
pub fn track_navigation(destination: &str, source: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "destination": destination,
                "source": source,
            });

            bindings::log_event(
                "navigation_click",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!("Analytics: Navigation - {} from {}", destination, source);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (destination, source);
    }
}

/// Track an outbound link click
pub fn track_outbound_link(url: &str, referer: &str, post_id: Option<&str>) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let mut params = json!({
                "url": url,
                "referer": referer,
            });

            if let Some(pid) = post_id {
                params["post_id"] = json!(pid);
            }

            bindings::log_event(
                "outbound_link",
                serde_wasm_bindgen::to_value(&params).unwrap(),
            );
            log::debug!("Analytics: Outbound link - {} from {}", url, referer);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, referer, post_id);
    }
}

/// Track a search event
pub fn track_search(query: &str, results_count: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            let params = json!({
                "search_query": query,
                "results_count": results_count,
            });

            bindings::log_event("search", serde_wasm_bindgen::to_value(&params).unwrap());
            log::debug!(
                "Analytics: Search - '{}' ({} results)",
                query,
                results_count
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (query, results_count);
    }
}

/// Track a custom event with arbitrary parameters
pub fn track_custom_event(event_name: &str, params: serde_json::Value) {
    #[cfg(target_arch = "wasm32")]
    {
        if bindings::is_analytics_available() {
            bindings::log_event(event_name, serde_wasm_bindgen::to_value(&params).unwrap());
            log::debug!("Analytics: Custom event - {}", event_name);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_name, params);
    }
}
