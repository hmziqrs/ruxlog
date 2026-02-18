//! Server functions for data fetching.
//!
//! Default mode uses API-backed endpoints.
//! `demo-static-content` mode serves local markdown content for SSG demos.

#[cfg(not(feature = "demo-static-content"))]
mod api_backed {
    #[allow(unused_imports)]
    use crate::router::Route;
    use dioxus::prelude::*;
    use oxstore::PaginatedList;
    #[allow(unused_imports)]
    use ruxlog_shared::store::PostListQuery;
    use ruxlog_shared::store::{Category, Post, Tag};

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
    pub async fn fetch_tags() -> Result<Vec<Tag>, ServerFnError> {
        let response = oxcore::http::get("/tag/v1/list")
            .send()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        if (200..300).contains(&response.status()) {
            response
                .json::<Vec<Tag>>()
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
    pub async fn fetch_categories() -> Result<Vec<Category>, ServerFnError> {
        let response = oxcore::http::get("/category/v1/list")
            .send()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        if (200..300).contains(&response.status()) {
            response
                .json::<Vec<Category>>()
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
        let response = oxcore::http::get(&format!("/tag/v1/view/{}", slug))
            .send()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if (200..300).contains(&response.status()) {
            response
                .json::<Tag>()
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
    pub async fn fetch_category_by_slug(slug: String) -> Result<Option<Category>, ServerFnError> {
        let response = oxcore::http::get(&format!("/category/v1/view/{}", slug))
            .send()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if (200..300).contains(&response.status()) {
            response
                .json::<Category>()
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

    #[server(endpoint = "static_routes", output = server_fn::codec::Json)]
    pub async fn static_routes() -> Result<Vec<String>, ServerFnError> {
        let mut routes = Route::static_routes()
            .into_iter()
            .map(|route| route.to_string())
            .collect::<Vec<_>>();
        routes.sort();
        routes.dedup();
        Ok(routes)
    }
}

#[cfg(feature = "demo-static-content")]
mod demo_static {
    #[allow(unused_imports)]
    use crate::{demo_content, router::Route};
    use dioxus::prelude::*;
    use oxstore::PaginatedList;
    use ruxlog_shared::store::{Category, Post, Tag};
    #[allow(unused_imports)]
    use std::collections::BTreeSet;

    pub async fn fetch_posts() -> Result<PaginatedList<Post>, ServerFnError> {
        Ok(demo_content::paginated(demo_content::content().posts()))
    }

    pub async fn fetch_post_by_slug(slug: String) -> Result<Option<Post>, ServerFnError> {
        Ok(demo_content::content().post_by_slug(&slug))
    }

    pub async fn fetch_tags() -> Result<Vec<Tag>, ServerFnError> {
        Ok(demo_content::content().tags().to_vec())
    }

    pub async fn fetch_categories() -> Result<Vec<Category>, ServerFnError> {
        Ok(demo_content::content().categories().to_vec())
    }

    pub async fn fetch_posts_by_tag(tag_id: i32) -> Result<PaginatedList<Post>, ServerFnError> {
        let posts = demo_content::content().posts_by_tag_id(tag_id);
        Ok(demo_content::paginated(&posts))
    }

    pub async fn fetch_posts_by_category(
        category_id: i32,
    ) -> Result<PaginatedList<Post>, ServerFnError> {
        let posts = demo_content::content().posts_by_category_id(category_id);
        Ok(demo_content::paginated(&posts))
    }

    pub async fn fetch_tag_by_slug(slug: String) -> Result<Option<Tag>, ServerFnError> {
        Ok(demo_content::content().tag_by_slug(&slug))
    }

    pub async fn fetch_category_by_slug(slug: String) -> Result<Option<Category>, ServerFnError> {
        Ok(demo_content::content().category_by_slug(&slug))
    }

    #[cfg(feature = "server")]
    #[server(endpoint = "static_routes", output = server_fn::codec::Json)]
    pub async fn static_routes() -> Result<Vec<String>, ServerFnError> {
        let mut routes = Route::static_routes()
            .into_iter()
            .map(|route| route.to_string())
            .collect::<BTreeSet<_>>();

        for route in demo_content::content().dynamic_routes() {
            routes.insert(route);
        }

        Ok(routes.into_iter().collect())
    }
}

#[cfg(not(feature = "demo-static-content"))]
pub use api_backed::*;
#[cfg(feature = "demo-static-content")]
pub use demo_static::*;
