use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_valid::Valid;

use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::post::{NewPost, Post, PostQuery, UpdatePost},
    services::auth::AuthSession,
    AppState,
};

use super::validator::{V1CreatePostPayload, V1PostQueryParams, V1UpdatePostPayload};

#[debug_handler]
pub async fn create_post(
    State(state): State<AppState>,
    auth: AuthSession,
    Valid(payload): Valid<Json<V1CreatePostPayload>>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let new_post = NewPost {
        title: payload.title,
        content: payload.content,
        author_id: user.id,
        published_at: payload.published_at,
        is_published: payload.is_published,
        slug: payload.slug,
        excerpt: payload.excerpt,
        featured_image_url: payload.featured_image_url,
        category_id: payload.category_id,
        view_count: 0,
        likes_count: 0,
    };

    match Post::create(&state.db_pool, new_post).await {
        Ok(post) => (StatusCode::CREATED, Json(json!(post))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to create post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn update_post(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
    Valid(payload): Valid<Json<V1UpdatePostPayload>>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let update_post = UpdatePost {
        title: payload.title.clone(),
        content: payload.content,
        author_id: Some(user.id),
        published_at: payload.published_at,
        updated_at: chrono::Utc::now().naive_utc(),
        is_published: payload.is_published,
        slug: payload.slug,
        excerpt: payload.excerpt,
        featured_image_url: payload.featured_image_url,
        category_id: payload.category_id,
        view_count: None,
        likes_count: None,
    };

    match Post::update(&state.db_pool, post_id, update_post).await {
        Ok(post) => (StatusCode::OK, Json(json!(post))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to update post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn delete_post(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
) -> impl IntoResponse {
    match Post::delete(&state.db_pool, post_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Post deleted successfully" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to delete post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn find_post_by_id(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
) -> impl IntoResponse {
    match Post::find_by_id(&state.db_pool, post_id).await {
        Ok(Some(post)) => (StatusCode::OK, Json(json!(post))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Post not found" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn find_all_posts(State(state): State<AppState>) -> impl IntoResponse {
    match Post::find_all(&state.db_pool).await {
        Ok(posts) => (StatusCode::OK, Json(json!(posts))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch posts",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn find_posts_with_query(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> impl IntoResponse {
    let post_query = PostQuery {
        pagination: query.page.and_then(|page| {
            query
                .per_page
                .map(|per_page| crate::db::models::post::Pagination { page, per_page })
        }),
        author_id: query.author_id,
        category_id: query.category_id,
        is_published: query.is_published,
        search: query.search,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    };

    match Post::find_posts_with_query(&state.db_pool, post_query).await {
        Ok(posts) => (StatusCode::OK, Json(json!(posts))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch posts",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn find_paginated_posts(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    match Post::find_paginated(&state.db_pool, page, per_page).await {
        Ok((posts, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": posts,
                "total": total,
                "page": page,
                "per_page": per_page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch paginated posts",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn find_published_posts(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    match Post::find_published_paginated(&state.db_pool, page, per_page).await {
        Ok((posts, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": posts,
                "total": total,
                "page": page,
                "per_page": per_page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch published posts",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn search_posts(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let search_term = query.search.unwrap_or_default();

    match Post::search_paginated(&state.db_pool, &search_term, page, per_page).await {
        Ok((posts, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": posts,
                "total": total,
                "page": page,
                "per_page": per_page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to search posts",
            })),
        )
            .into_response(),
    }
}
