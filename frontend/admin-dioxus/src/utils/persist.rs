use bevy_pkv::PkvStore;
#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, Storage};
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Global persistent key-value store. On web, this uses localStorage under the hood.
// For desktop/native, bevy_pkv uses a lightweight embedded store.
pub static PKV: Lazy<Mutex<PkvStore>> =
    Lazy::new(|| Mutex::new(PkvStore::new("Ruxlog", "AdminDioxus")));

const THEME_KEY: &str = "theme"; // values: "dark" | "light"

pub fn get_theme() -> Option<String> {
    PKV.lock().ok()?.get::<String>(THEME_KEY).ok()
}

pub fn set_theme(theme: &str) {
    // Best-effort; ignore errors to avoid breaking UI interactions.
    if let Ok(mut store) = PKV.lock() {
        let _ = store.set_string(THEME_KEY, theme);
    }
    // Mirror the value into window.localStorage so we can apply theme pre-hydration in index.html
    #[cfg(target_arch = "wasm32")]
    {
        let _ = LocalStorage::set(THEME_KEY, theme);
    }
}
