use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn use_unique_id() -> Signal<String> {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[allow(unused_mut)]
    let mut initial_value = use_hook(|| {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("ox-id-{id}")
    });

    use_signal(|| initial_value)
}
