use dioxus::prelude::*;
use oxstore::PaginatedList;
use ruxlog_shared::{Post, PostListQuery};

#[cfg(feature = "server")]
use crate::server::{fetch_post_by_id, fetch_post_by_slug, fetch_posts, fetch_posts_with_query};

/// Fetch all posts with SSR support
///
/// # Example
/// ```rust
/// let posts = use_post_list()?;
/// let data = posts().ok()?;
/// ```
#[cfg(feature = "server")]
pub fn use_post_list() -> Resource<Result<PaginatedList<Post>, ServerFnError>> {
    use_server_future(fetch_posts).expect("Failed to create server future for post list")
}

/// Fetch posts with query parameters
///
/// # Example
/// ```rust
/// let query = PostListQuery {
///     category_id: Some(1),
///     ..Default::default()
/// };
/// let posts = use_post_list_with_query(query)?;
/// ```
#[cfg(feature = "server")]
pub fn use_post_list_with_query(
    query: PostListQuery,
) -> Resource<Result<PaginatedList<Post>, ServerFnError>> {
    use_server_future(move || fetch_posts_with_query(query.clone()))
        .expect("Failed to create server future for post list with query")
}

/// Get a single post by ID
///
/// # Example
/// ```rust
/// let post = use_post_by_id(1)?;
/// let data = post().ok()?.flatten()?;
/// ```
#[cfg(feature = "server")]
pub fn use_post_by_id(id: i32) -> Resource<Result<Option<Post>, ServerFnError>> {
    use_server_future(move || fetch_post_by_id(id))
        .expect("Failed to create server future for post by id")
}

/// Get a single post by slug
///
/// # Example
/// ```rust
/// let post = use_post_by_slug("my-post-slug".to_string())?;
/// let data = post().ok()?.flatten()?;
/// ```
#[cfg(feature = "server")]
pub fn use_post_by_slug(slug: String) -> Resource<Result<Option<Post>, ServerFnError>> {
    use_server_future(move || fetch_post_by_slug(slug.clone()))
        .expect("Failed to create server future for post by slug")
}

// ============================================================================
// Client-only fallbacks (when server feature is disabled)
// ============================================================================

#[cfg(not(feature = "server"))]
pub fn use_post_list() -> Signal<Option<PaginatedList<Post>>> {
    use ruxlog_shared::use_post;

    let posts_store = use_post();
    let data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            posts_store.list().await;
            let frame = posts_store.list.read();
            if let Some(posts_data) = &frame.data {
                data.set(Some(posts_data.clone()));
            }
        });
    });

    data
}

#[cfg(not(feature = "server"))]
pub fn use_post_list_with_query(query: PostListQuery) -> Signal<Option<PaginatedList<Post>>> {
    use ruxlog_shared::use_post;

    let posts_store = use_post();
    let data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            posts_store.list_with_query(query.clone()).await;
            let frame = posts_store.list.read();
            if let Some(posts_data) = &frame.data {
                data.set(Some(posts_data.clone()));
            }
        });
    });

    data
}

#[cfg(not(feature = "server"))]
pub fn use_post_by_id(id: i32) -> Signal<Option<Post>> {
    use ruxlog_shared::use_post;

    let posts_store = use_post();
    let data = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            posts_store.view_by_id(id).await;
            let frame = posts_store.view.read();
            if let Some(post_frame) = frame.get(&id) {
                if let Some(post_data) = &post_frame.data {
                    data.set(Some(post_data.clone()));
                }
            }
        });
    });

    data
}

#[cfg(not(feature = "server"))]
pub fn use_post_by_slug(slug: String) -> Signal<Option<Post>> {
    use ruxlog_shared::use_post;

    let posts_store = use_post();
    let data = use_signal(|| None);

    use_effect(move || {
        let slug_clone = slug.clone();
        spawn(async move {
            posts_store.view(&slug_clone).await;
            let frame = posts_store.view.read();
            // Find the post in the HashMap by checking if any value matches the slug
            let post = frame.values().find_map(|f| {
                f.data
                    .as_ref()
                    .filter(|post| post.slug == slug_clone)
                    .cloned()
            });
            data.set(post);
        });
    });

    data
}
