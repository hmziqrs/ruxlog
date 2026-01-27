//! Firebase Analytics JS bindings via wasm-bindgen
//!
//! This module provides low-level JavaScript interop for Firebase Analytics SDK.
//! Only available on wasm32 target.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/assets/firebase-init.js")]
extern "C" {
    /// Initialize Firebase with the provided configuration
    #[wasm_bindgen(js_name = initFirebase)]
    pub fn init_firebase(
        api_key: &str,
        auth_domain: &str,
        project_id: &str,
        storage_bucket: &str,
        messaging_sender_id: &str,
        app_id: &str,
        measurement_id: &str,
    ) -> bool;

    /// Check if Firebase Analytics is available
    #[wasm_bindgen(js_name = isAnalyticsAvailable)]
    pub fn is_analytics_available() -> bool;

    /// Log a custom event with optional parameters
    #[wasm_bindgen(js_name = logAnalyticsEvent)]
    pub fn log_event(event_name: &str, params: JsValue);

    /// Log a page view event
    #[wasm_bindgen(js_name = logPageView)]
    pub fn log_page_view(page_path: &str, page_title: &str);

    /// Set a user property
    #[wasm_bindgen(js_name = setUserProperty)]
    pub fn set_user_property(name: &str, value: &str);

    /// Set the current screen/page name
    #[wasm_bindgen(js_name = setCurrentScreen)]
    pub fn set_current_screen(screen_name: &str);
}

// No-op stubs for non-wasm targets
#[cfg(not(target_arch = "wasm32"))]
pub fn init_firebase(
    _api_key: &str,
    _auth_domain: &str,
    _project_id: &str,
    _storage_bucket: &str,
    _messaging_sender_id: &str,
    _app_id: &str,
    _measurement_id: &str,
) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_analytics_available() -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log_event(_event_name: &str, _params: ()) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn log_page_view(_page_path: &str, _page_title: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_user_property(_name: &str, _value: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_current_screen(_screen_name: &str) {}
