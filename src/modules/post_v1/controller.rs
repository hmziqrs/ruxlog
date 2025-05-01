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
    db::sea_models::post, 
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    modules::post_v1::validator::V1UpdatePostPayload, 
    services::auth::AuthSession, 
    AppState,
};

use super::validator::{V1CreatePostPayload, V1PostQueryParams};

#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1CreatePostPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let new_post = payload.0.into_new_post(user.id);

    match post::Entity::create(&state.sea_db, new_post).await {
        Ok(post) => Ok((StatusCode::CREATED, Json(json!(post)))),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn find_by_id_or_slug(
    State(state): State<AppState>,
    Path(slug_or_id): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let query = match slug_or_id.parse::<i32>() {
        Ok(id) => post::Entity::find_by_id_or_slug(&state.sea_db, Some(id), None).await,
        Err(_) => post::Entity::find_by_id_or_slug(&state.sea_db, None, Some(slug_or_id)).await,
    };

    match query {
        Ok(Some(post)) => Ok((StatusCode::OK, Json(json!(post)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Post not found")),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to fetch post")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    payload: ValidatedJson<V1UpdatePostPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let update_post = payload.0.into_update_post();

    match post::Entity::update(&state.sea_db, post_id, update_post).await {
        Ok(Some(post)) => Ok((StatusCode::OK, Json(json!(post)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Post does not exist")),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to update post")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match post::Entity::delete(&state.sea_db, post_id).await {
        Ok(1) => Ok((StatusCode::OK, Json(json!({ "message": "Post deleted successfully" })))),
        Ok(0) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Post does not exist")),
        Ok(_) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Internal server error occurred while deleting post")),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to delete post")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn find_published_posts(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let page = query.page.unwrap_or(1);

    match post::Entity::find_published_paginated(&state.sea_db, page).await {
        Ok((posts, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": posts,
                "total": total,
                "per_page": post::Entity::PER_PAGE as u64,
                "page": page,
            })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to fetch published posts")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn track_view(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user_id: Option<i32> = auth.user.map(|user| user.id);
    match post::Entity::increment_view_count(&state.sea_db, post_id, user_id, None, None).await {
        Ok(_) => Ok((StatusCode::OK, Json(json!({ "message": "View tracked successfully" })))),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to track view")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn sitemap(State(state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    match post::Entity::sitemap(&state.sea_db).await {
        Ok(posts) => Ok((StatusCode::OK, Json(posts))),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to fetch posts")
            .with_details(err.to_string())),
    }
}
