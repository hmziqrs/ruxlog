//! Server functions for data fetching (API-backed).

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

pub use api_backed::*;
