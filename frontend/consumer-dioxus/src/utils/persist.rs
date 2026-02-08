#[cfg(target_arch = "wasm32")]
use bevy_pkv::PkvStore;
#[cfg(target_arch = "wasm32")]
use once_cell::sync::Lazy;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

// Global persistent key-value store. On web, this uses localStorage under the hood.
#[cfg(target_arch = "wasm32")]
pub static PKV: Lazy<Mutex<PkvStore>> =
    Lazy::new(|| Mutex::new(PkvStore::new("Ruxlog", "ConsumerDioxus")));

const THEME_KEY: &str = "theme"; // values: "dark" | "light"

#[cfg(target_arch = "wasm32")]
pub fn get_theme() -> Option<String> {
    PKV.lock().ok()?.get::<String>(THEME_KEY).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_theme() -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn set_theme(theme: &str) {
    if let Ok(mut store) = PKV.lock() {
        let _ = store.set_string(THEME_KEY, theme);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_theme(_theme: &str) {
    // No-op on server
}
