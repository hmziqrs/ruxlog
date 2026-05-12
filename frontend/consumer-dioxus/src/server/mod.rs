use dioxus::prelude::*;
use oxstore::PaginatedList;
use ruxlog_shared::{Category, Post, PostListQuery, Tag};

mod api_backed {
    use super::*;
    use ruxlog_shared::{use_categories, use_post, use_tag};

    #[server]
    pub async fn fetch_posts() -> Result<PaginatedList<Post>, ServerFnError> {
        use_post().list().await;
        let frame = use_post().list.read();
        frame
            .data
            .clone()
            .ok_or_else(|| ServerFnError::new("No posts data available"))
    }

    #[server]
    pub async fn fetch_post_by_id(id: i32) -> Result<Option<Post>, ServerFnError> {
        use_post().view_by_id(id).await;
        let frame = use_post().view.read();
        Ok(frame.get(&id).and_then(|f| f.data.clone()))
    }

    #[server]
    pub async fn fetch_post_by_slug(slug: String) -> Result<Option<Post>, ServerFnError> {
        use_post().view(&slug).await;
        let frame = use_post().view.read();
        Ok(frame
            .values()
            .find_map(|f| f.data.as_ref().filter(|post| post.slug == slug).cloned()))
    }

    #[server]
    pub async fn fetch_posts_with_query(
        query: PostListQuery,
    ) -> Result<PaginatedList<Post>, ServerFnError> {
        use_post().list_with_query(query).await;
        let frame = use_post().list.read();
        frame
            .data
            .clone()
            .ok_or_else(|| ServerFnError::new("No posts data available"))
    }

    #[server]
    pub async fn fetch_categories() -> Result<PaginatedList<Category>, ServerFnError> {
        use_categories().list_all().await;
        let frame = use_categories().list.read();
        frame
            .data
            .clone()
            .ok_or_else(|| ServerFnError::new("No categories data available"))
    }

    #[server]
    pub async fn fetch_category_by_id(id: i32) -> Result<Option<Category>, ServerFnError> {
        use_categories().view(id).await;
        let frame = use_categories().view.read();
        Ok(frame.get(&id).and_then(|f| f.data.clone()))
    }

    #[server]
    pub async fn fetch_tags() -> Result<PaginatedList<Tag>, ServerFnError> {
        use_tag().list_all().await;
        let frame = use_tag().list.read();
        frame
            .data
            .clone()
            .ok_or_else(|| ServerFnError::new("No tags data available"))
    }

    #[server]
    pub async fn fetch_tag_by_id(id: i32) -> Result<Option<Tag>, ServerFnError> {
        use_tag().view(id).await;
        let frame = use_tag().view.read();
        Ok(frame.get(&id).and_then(|f| f.data.clone()))
    }
}

pub use api_backed::*;
