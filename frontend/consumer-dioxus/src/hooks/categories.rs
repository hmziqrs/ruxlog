use dioxus::prelude::*;
use oxstore::PaginatedList;
use ruxlog_shared::Category;

#[cfg(feature = "server")]
use crate::server::{fetch_categories, fetch_category_by_id};

/// Fetch all categories with SSR support
///
/// # Example
/// ```rust
/// let categories = use_categories_list()?;
/// let data = categories().ok()?;
/// ```
#[cfg(feature = "server")]
pub fn use_categories_list() -> Resource<Result<PaginatedList<Category>, ServerFnError>> {
    use_server_future(fetch_categories).expect("Failed to create server future for categories")
}

/// Get a single category by ID
#[cfg(feature = "server")]
pub fn use_category_by_id(id: i32) -> Resource<Result<Option<Category>, ServerFnError>> {
    use_server_future(move || fetch_category_by_id(id))
        .expect("Failed to create server future for category by id")
}

/// Find category by slug from fetched list
///
/// # Example
/// ```rust
/// let category = use_category_by_slug("rust".to_string());
/// if let Some(cat) = category() {
///     // Use category
/// }
/// ```
#[cfg(feature = "server")]
pub fn use_category_by_slug(slug: String) -> Memo<Option<Category>> {
    let list = use_categories_list();

    use_memo(move || {
        if let Some(result) = list.peek().as_ref() {
            if let Ok(paginated) = result.as_ref() {
                return paginated.data.iter().find(|c| c.slug == slug).cloned();
            }
        }
        None
    })
}

// ============================================================================
// Client-only fallbacks (when server feature is disabled)
// ============================================================================

#[cfg(not(feature = "server"))]
pub fn use_categories_list() -> Signal<Option<PaginatedList<Category>>> {
    use ruxlog_shared::use_categories;

    let categories_store = use_categories();
    let data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            categories_store.list_all().await;
            let frame = categories_store.list.read();
            if let Some(categories_data) = &frame.data {
                data.set(Some(categories_data.clone()));
            }
        });
    });

    data
}

#[cfg(not(feature = "server"))]
pub fn use_category_by_id(id: i32) -> Signal<Option<Category>> {
    use ruxlog_shared::use_categories;

    let categories_store = use_categories();
    let data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            categories_store.view(id).await;
            let frame = categories_store.view.read();
            if let Some(category_frame) = frame.get(&id) {
                if let Some(category_data) = &category_frame.data {
                    data.set(Some(category_data.clone()));
                }
            }
        });
    });

    data
}

#[cfg(not(feature = "server"))]
pub fn use_category_by_slug(slug: String) -> Signal<Option<Category>> {
    use ruxlog_shared::use_categories;

    let categories_store = use_categories();
    let data = use_signal(|| None);

    use_effect(move || {
        let slug_clone = slug.clone();
        spawn(async move {
            categories_store.list_all().await;
            let frame = categories_store.list.read();
            if let Some(categories_data) = &frame.data {
                let category = categories_data
                    .data
                    .iter()
                    .find(|c| c.slug == slug_clone)
                    .cloned();
                data.set(category);
            }
        });
    });

    data
}
