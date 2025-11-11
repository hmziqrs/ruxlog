use dioxus::prelude::*;

use crate::store::{use_user, User};

#[derive(Clone)]
pub struct UserViewState {
    pub is_loading: bool,
    pub is_failed: bool,
    pub message: Option<String>,
    pub user: Option<User>,
}

pub fn use_user_view(id: i32) -> UserViewState {
    let users = use_user();

    // Trigger fetch on mount/when needed
    {
        let users = users;
        use_effect(move || {
            let view_map = users.view.read();
            let needs_fetch = match view_map.get(&id) {
                None => true,
                Some(frame) => frame.is_init(),
            };
            if needs_fetch {
                spawn(async move {
                    let users = use_user();
                    users.view(id).await;
                });
            }
        });
    }

    let view_map = users.view.read();
    let frame = view_map.get(&id);

    let is_loading = frame.map(|f| f.is_loading()).unwrap_or(true);
    let is_failed = frame.map(|f| f.is_failed()).unwrap_or(false);
    let message = frame.and_then(|f| f.error_message());
    let user = frame.and_then(|f| f.data.clone()).into_iter().next();

    UserViewState {
        is_loading,
        is_failed,
        message,
        user,
    }
}
