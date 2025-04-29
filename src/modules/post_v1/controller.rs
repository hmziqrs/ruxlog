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
    db::sea_models::post, extractors::ValidatedJson,
    modules::post_v1::validator::V1UpdatePostPayload, services::auth::AuthSession, AppState,
};

use super::validator::{V1CreatePostPayload, V1PostQueryParams};

#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1CreatePostPayload>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let new_post = payload.0.into_new_post(user.id);

    match post::Entity::create(&state.sea_db, new_post).await {
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
pub async fn find_by_id_or_slug(
    State(state): State<AppState>,
    Path(slug_or_id): Path<String>,
) -> impl IntoResponse {
    let query = match slug_or_id.parse::<i32>() {
        Ok(id) => post::Entity::find_by_id_or_slug(&state.sea_db, Some(id), None).await,
        Err(_) => post::Entity::find_by_id_or_slug(&state.sea_db, None, Some(slug_or_id)).await,
    };

    match query {
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
pub async fn update(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
    payload: ValidatedJson<V1UpdatePostPayload>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let update_post = payload.0.into_update_post(user.id);

    match post::Entity::update(&state.sea_db, post_id, user, update_post).await {
        Ok(Some(post)) => (StatusCode::OK, Json(json!(post))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "requested failed",
                "message": "Post does not exist",
            })),
        )
            .into_response(),
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
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    match post::Entity::delete(&state.sea_db, user, post_id).await {
        Ok(1) => (
            StatusCode::OK,
            Json(json!({ "message": "Post deleted successfully" })),
        )
            .into_response(),
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Post does not exist",
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "unexpected result",
                "message": "Internal server error occurred while deleting post",
            })),
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
pub async fn find_posts_with_query(
    State(state): State<AppState>,
    auth: AuthSession,
    query: ValidatedJson<V1PostQueryParams>,
) -> impl IntoResponse {
    let post_query = query.0.into_post_query();

    match post::Entity::find_posts_with_query(&state.sea_db, post_query, auth.user.unwrap()).await {
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
pub async fn find_published_posts(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);

    match post::Entity::find_published_paginated(&state.sea_db, page).await {
        Ok((posts, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": posts,
                "total": total,
                "per_page": post::Entity::PER_PAGE,
                "page": page,
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
pub async fn track_view(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
) -> impl IntoResponse {
    let user_id: Option<i32> = auth.user.map(|user| user.id);
    match post::Entity::increment_view_count(&state.sea_db, post_id, user_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "View tracked successfully" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to track view",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn sitemap(State(state): State<AppState>) -> impl IntoResponse {
    match post::Entity::sitemap(&state.sea_db).await {
        Ok(posts) => (StatusCode::OK, Json(posts)).into_response(),
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
