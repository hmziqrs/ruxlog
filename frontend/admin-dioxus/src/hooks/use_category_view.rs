use dioxus::prelude::*;

use crate::store::{use_categories, Category};

#[derive(Clone)]
pub struct CategoryViewState {
    pub is_loading: bool,
    pub is_failed: bool,
    pub message: Option<String>,
    pub category: Option<Category>,
}

pub fn use_category_view(id: i32) -> CategoryViewState {
    let categories = use_categories();

    // Trigger fetch on mount/when needed
    {
        let categories = categories;
        use_effect(move || {
            let view_map = categories.view.read();
            let needs_fetch = match view_map.get(&id) {
                None => true,
                Some(frame) => frame.is_init(),
            };
            if needs_fetch {
                spawn(async move {
                    let c = use_categories();
                    c.view(id).await;
                });
            }
        });
    }

    let view_map = categories.view.read();
    let frame = view_map.get(&id);

    let is_loading = frame.map(|f| f.is_loading()).unwrap_or(true);
    let is_failed = frame.map(|f| f.is_failed()).unwrap_or(false);
    let message = frame.and_then(|f| f.error_message());
    let category = frame.and_then(|f| f.data.clone());

    CategoryViewState {
        is_loading,
        is_failed,
        message,
        category,
    }
}
