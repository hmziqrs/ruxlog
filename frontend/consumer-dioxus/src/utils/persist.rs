use bevy_pkv::PkvStore;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Global persistent key-value store. On web, this uses localStorage under the hood.
pub static PKV: Lazy<Mutex<PkvStore>> =
    Lazy::new(|| Mutex::new(PkvStore::new("Ruxlog", "ConsumerDioxus")));

const THEME_KEY: &str = "theme"; // values: "dark" | "light"

pub fn get_theme() -> Option<String> {
    PKV.lock().ok()?.get::<String>(THEME_KEY).ok()
}

pub fn set_theme(theme: &str) {
    if let Ok(mut store) = PKV.lock() {
        let _ = store.set_string(THEME_KEY, theme);
    }
}
