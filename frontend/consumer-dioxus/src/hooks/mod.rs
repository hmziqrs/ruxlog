use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn use_unique_id() -> Signal<String> {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[allow(unused_mut)]
    let mut initial_value = use_hook(|| {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let id_str = format!("dxc-{id}");
        id_str
    });

    // fullstack! macro might need server feature or similar, but for now let's keep it simple or match admin-dioxus
    // admin-dioxus uses fullstack! macro.
    // consumer-dioxus has dioxus with "fullstack" feature in my previous view, but I changed it to "router" in step 73.
    // Wait, step 73 I set features = ["router"].
    // admin-dioxus has features = ["router"].
    // But admin-dioxus uses `fullstack!` macro in `use_unique_id`.
    // `fullstack!` comes from `dioxus` prelude if fullstack feature is enabled?
    // admin-dioxus Cargo.toml:
    // dioxus = { version = "0.7.1", features = ["router"] }
    // It does NOT have "fullstack" feature enabled explicitly in dependencies, but maybe via `dioxus/web` default?
    // [features] default = ["web"]. web = ["dioxus/web"].
    
    // Let's check if `fullstack!` works. If not, I'll remove it for now as we are client-side mostly.
    // Actually, `admin-dioxus` uses `fullstack!`.
    
    // For now, I will just return the signal.
    use_signal(|| initial_value)
}
