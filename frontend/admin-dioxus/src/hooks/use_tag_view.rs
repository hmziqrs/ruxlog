use dioxus::prelude::*;

use crate::store::{use_tag, Tag};

#[derive(Clone)]
pub struct TagViewState {
    pub is_loading: bool,
    pub is_failed: bool,
    pub message: Option<String>,
    pub tag: Option<Tag>,
}

pub fn use_tag_view(id: i32) -> TagViewState {
    let tags = use_tag();

    // Trigger fetch on mount/when needed
    {
        let tags = tags;
        use_effect(move || {
            let view_map = tags.view.read();
            let needs_fetch = match view_map.get(&id) {
                None => true,
                Some(frame) => frame.is_init(),
            };
            if needs_fetch {
                spawn(async move {
                    let tags = use_tag();
                    tags.view(id).await;
                });
            }
        });
    }

    let view_map = tags.view.read();
    let frame = view_map.get(&id);

    let is_loading = frame.map(|f| f.is_loading()).unwrap_or(true);
    let is_failed = frame.map(|f| f.is_failed()).unwrap_or(false);
    let message = frame.and_then(|f| f.error_message());
    let tag = frame.and_then(|f| f.data.clone()).into_iter().next();

    TagViewState {
        is_loading,
        is_failed,
        message,
        tag,
    }
}
