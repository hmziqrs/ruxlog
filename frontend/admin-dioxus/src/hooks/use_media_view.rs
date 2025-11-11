use dioxus::prelude::*;

use crate::store::{use_media, Media};

#[derive(Clone)]
pub struct MediaViewState {
    pub is_loading: bool,
    pub is_failed: bool,
    pub message: Option<String>,
    pub media: Option<Media>,
}

pub fn use_media_view(id: i32) -> MediaViewState {
    let media_state = use_media();

    // Trigger fetch on mount/when needed
    {
        let media_state = media_state;
        use_effect(move || {
            let view_map = media_state.view.read();
            let needs_fetch = match view_map.get(&id) {
                None => true,
                Some(frame) => frame.is_init(),
            };
            if needs_fetch {
                spawn(async move {
                    let media_state = use_media();
                    media_state.view(id).await;
                });
            }
        });
    }

    let view_map = media_state.view.read();
    let frame = view_map.get(&id);

    let is_loading = frame.map(|f| f.is_loading()).unwrap_or(true);
    let is_failed = frame.map(|f| f.is_failed()).unwrap_or(false);
    let message = frame.and_then(|f| f.error_message());
    let media = frame.and_then(|f| f.data.clone());

    MediaViewState {
        is_loading,
        is_failed,
        message,
        media,
    }
}
