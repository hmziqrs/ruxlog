use dioxus::prelude::*;
use oxstore::PaginatedList;
use ruxlog_shared::{Category, Post, PostListQuery, Tag};

#[cfg(not(feature = "demo-static-content"))]
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

#[cfg(feature = "demo-static-content")]
mod demo_static {
    use super::*;
    use crate::demo_content;

    fn paginated<T: Clone>(items: Vec<T>, page: u64) -> PaginatedList<T> {
        PaginatedList {
            total: items.len() as u64,
            per_page: std::cmp::max(items.len(), 1) as u64,
            page,
            data: items,
        }
    }

    fn filter_posts(query: &PostListQuery) -> Vec<Post> {
        let mut posts = demo_content::content().posts().to_vec();

        if let Some(author_id) = query.author_id {
            posts.retain(|post| post.author.id == author_id);
        }

        if let Some(category_id) = query.category_id {
            posts.retain(|post| post.category.id == category_id);
        }

        if let Some(tag_ids) = &query.tag_ids {
            if !tag_ids.is_empty() {
                posts.retain(|post| post.tags.iter().any(|tag| tag_ids.contains(&tag.id)));
            }
        }

        if let Some(status) = &query.status {
            posts.retain(|post| &post.status == status);
        }

        if let Some(search) = query.search.as_deref() {
            let needle = search.to_lowercase();
            posts.retain(|post| {
                post.title.to_lowercase().contains(&needle)
                    || post
                        .excerpt
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&needle)
            });
        }

        if let Some(title) = query.title.as_deref() {
            let needle = title.to_lowercase();
            posts.retain(|post| post.title.to_lowercase().contains(&needle));
        }

        posts
    }

    #[server]
    pub async fn fetch_posts() -> Result<PaginatedList<Post>, ServerFnError> {
        Ok(demo_content::paginated(demo_content::content().posts()))
    }

    #[server]
    pub async fn fetch_post_by_id(id: i32) -> Result<Option<Post>, ServerFnError> {
        Ok(demo_content::content()
            .posts()
            .iter()
            .find(|post| post.id == id)
            .cloned())
    }

    #[server]
    pub async fn fetch_post_by_slug(slug: String) -> Result<Option<Post>, ServerFnError> {
        Ok(demo_content::content().post_by_slug(&slug))
    }

    #[server]
    pub async fn fetch_posts_with_query(
        query: PostListQuery,
    ) -> Result<PaginatedList<Post>, ServerFnError> {
        let page = query.page.unwrap_or(1);
        Ok(paginated(filter_posts(&query), page))
    }

    #[server]
    pub async fn fetch_categories() -> Result<PaginatedList<Category>, ServerFnError> {
        Ok(paginated(demo_content::content().categories().to_vec(), 1))
    }

    #[server]
    pub async fn fetch_category_by_id(id: i32) -> Result<Option<Category>, ServerFnError> {
        Ok(demo_content::content()
            .categories()
            .iter()
            .find(|category| category.id == id)
            .cloned())
    }

    #[server]
    pub async fn fetch_tags() -> Result<PaginatedList<Tag>, ServerFnError> {
        Ok(paginated(demo_content::content().tags().to_vec(), 1))
    }

    #[server]
    pub async fn fetch_tag_by_id(id: i32) -> Result<Option<Tag>, ServerFnError> {
        Ok(demo_content::content()
            .tags()
            .iter()
            .find(|tag| tag.id == id)
            .cloned())
    }
}

#[cfg(not(feature = "demo-static-content"))]
pub use api_backed::*;
#[cfg(feature = "demo-static-content")]
pub use demo_static::*;
