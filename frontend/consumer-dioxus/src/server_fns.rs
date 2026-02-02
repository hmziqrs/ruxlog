//! Server functions for SSR data fetching.
//!
//! These functions run on the server during SSR and their results are serialized
//! for hydration on the client.

use dioxus::prelude::*;
use oxstore::PaginatedList;
use ruxlog_shared::store::{Category, Post, PostListQuery, Tag};

#[server]
pub async fn fetch_posts() -> Result<PaginatedList<Post>, ServerFnError> {
    let response = oxcore::http::post("/post/v1/list/published", &serde_json::json!({}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        response
            .json::<PaginatedList<Post>>()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_post_by_slug(slug: String) -> Result<Option<Post>, ServerFnError> {
    let response = oxcore::http::post(&format!("/post/v1/view/{}", slug), &())
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if response.status() == 404 {
        return Ok(None);
    }

    if (200..300).contains(&response.status()) {
        response
            .json::<Post>()
            .await
            .map(Some)
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_tags() -> Result<PaginatedList<Tag>, ServerFnError> {
    let response = oxcore::http::post("/tags/v1/query", &serde_json::json!({}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        response
            .json::<PaginatedList<Tag>>()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_categories() -> Result<PaginatedList<Category>, ServerFnError> {
    let response = oxcore::http::post("/categories/v1/query", &serde_json::json!({}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        response
            .json::<PaginatedList<Category>>()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_posts_by_tag(tag_id: i32) -> Result<PaginatedList<Post>, ServerFnError> {
    let query = PostListQuery {
        page: Some(1),
        tag_ids: Some(vec![tag_id]),
        ..Default::default()
    };

    let response = oxcore::http::post("/post/v1/list/published", &query)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        response
            .json::<PaginatedList<Post>>()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_posts_by_category(
    category_id: i32,
) -> Result<PaginatedList<Post>, ServerFnError> {
    let query = PostListQuery {
        page: Some(1),
        category_id: Some(category_id),
        ..Default::default()
    };

    let response = oxcore::http::post("/post/v1/list/published", &query)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        response
            .json::<PaginatedList<Post>>()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_tag_by_slug(slug: String) -> Result<Option<Tag>, ServerFnError> {
    // First fetch all tags, then find the one with matching slug
    let response = oxcore::http::post("/tags/v1/query", &serde_json::json!({}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        let tags: PaginatedList<Tag> = response
            .json()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(tags.data.into_iter().find(|t| t.slug == slug))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}

#[server]
pub async fn fetch_category_by_slug(slug: String) -> Result<Option<Category>, ServerFnError> {
    // First fetch all categories, then find the one with matching slug
    let response = oxcore::http::post("/categories/v1/query", &serde_json::json!({}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if (200..300).contains(&response.status()) {
        let categories: PaginatedList<Category> = response
            .json()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(categories.data.into_iter().find(|c| c.slug == slug))
    } else {
        Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )))
    }
}
