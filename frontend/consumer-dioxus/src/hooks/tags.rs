use dioxus::prelude::*;
use oxstore::PaginatedList;
use ruxlog_shared::Tag;

#[cfg(all(feature = "server", not(feature = "demo-static-content")))]
use crate::server::{fetch_tag_by_id, fetch_tags};

/// Fetch all tags with SSR support
///
/// # Example
/// ```rust
/// let tags = use_tags_list()?;
/// let data = tags().ok()?;
/// ```
#[cfg(all(feature = "server", not(feature = "demo-static-content")))]
pub fn use_tags_list() -> Resource<Result<PaginatedList<Tag>, ServerFnError>> {
    use_server_future(fetch_tags).expect("Failed to create server future for tags")
}

/// Get a single tag by ID
#[cfg(all(feature = "server", not(feature = "demo-static-content")))]
pub fn use_tag_by_id(id: i32) -> Resource<Result<Option<Tag>, ServerFnError>> {
    use_server_future(move || fetch_tag_by_id(id))
        .expect("Failed to create server future for tag by id")
}

/// Find tag by slug from fetched list
///
/// # Example
/// ```rust
/// let tag = use_tag_by_slug("rust".to_string());
/// if let Some(t) = tag() {
///     // Use tag
/// }
/// ```
#[cfg(all(feature = "server", not(feature = "demo-static-content")))]
pub fn use_tag_by_slug(slug: String) -> Memo<Option<Tag>> {
    let list = use_tags_list();

    use_memo(move || {
        if let Some(result) = list.peek().as_ref() {
            if let Ok(paginated) = result.as_ref() {
                return paginated.data.iter().find(|t| t.slug == slug).cloned();
            }
        }
        None
    })
}

// ============================================================================
// Client-only fallbacks (when server feature is disabled)
// ============================================================================

#[cfg(any(not(feature = "server"), feature = "demo-static-content"))]
pub fn use_tags_list() -> Signal<Option<PaginatedList<Tag>>> {
    use ruxlog_shared::use_tag;

    let tags_store = use_tag();
    let mut data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            tags_store.list_all().await;
            let frame = tags_store.list.read();
            if let Some(tags_data) = &frame.data {
                data.set(Some(tags_data.clone()));
            }
        });
    });

    data
}

#[cfg(any(not(feature = "server"), feature = "demo-static-content"))]
pub fn use_tag_by_id(id: i32) -> Signal<Option<Tag>> {
    use ruxlog_shared::use_tag;

    let tags_store = use_tag();
    let mut data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            tags_store.view(id).await;
            let frame = tags_store.view.read();
            if let Some(tag_frame) = frame.get(&id) {
                if let Some(tag_data) = &tag_frame.data {
                    data.set(Some(tag_data.clone()));
                }
            }
        });
    });

    data
}

#[cfg(any(not(feature = "server"), feature = "demo-static-content"))]
pub fn use_tag_by_slug(slug: String) -> Signal<Option<Tag>> {
    use ruxlog_shared::use_tag;

    let tags_store = use_tag();
    let mut data = use_signal(|| None);

    use_effect(move || {
        let slug_clone = slug.clone();
        spawn(async move {
            tags_store.list_all().await;
            let frame = tags_store.list.read();
            if let Some(tags_data) = &frame.data {
                let tag = tags_data
                    .data
                    .iter()
                    .find(|t| t.slug == slug_clone)
                    .cloned();
                data.set(tag);
            }
        });
    });

    data
}
