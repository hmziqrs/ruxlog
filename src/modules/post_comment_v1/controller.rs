use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use crate::{db::models::post_comment::PostComment, services::auth::AuthSession, AppState};

use super::validator::{
    V1CreatePostCommentPayload, V1PostCommentQueryParams, V1UpdatePostCommentPayload,
};

#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: Valid<Json<V1CreatePostCommentPayload>>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let new_comment = payload.into_inner().0.into_new_post_comment(user.id);

    match PostComment::create(&state.db_pool, new_comment).await {
        Ok(comment) => (StatusCode::CREATED, Json(json!(comment))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to create comment",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
    payload: Valid<Json<V1UpdatePostCommentPayload>>,
) -> impl IntoResponse {
    let update_comment = payload.into_inner().0.into_update_post_comment();

    match PostComment::update(&state.db_pool, comment_id, update_comment).await {
        Ok(comment) => (StatusCode::OK, Json(json!(comment))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to update comment",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    Path(comment_id): Path<i32>,
) -> impl IntoResponse {
    match PostComment::delete(&state.db_pool, comment_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Comment deleted successfully" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to delete comment",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn list_all(State(state): State<AppState>) -> impl IntoResponse {
    match PostComment::list_all(&state.db_pool).await {
        Ok(comments) => (StatusCode::OK, Json(json!(comments))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch comments",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn list_paginated(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostCommentQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);

    match PostComment::list_paginated(&state.db_pool, page).await {
        Ok((comments, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch paginated comments",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn list_with_query(
    State(state): State<AppState>,
    query: Valid<Query<V1PostCommentQueryParams>>,
) -> impl IntoResponse {
    let post_comment_query = query.into_inner().0.into_post_comment_query();

    match PostComment::list_with_query(&state.db_pool, post_comment_query).await {
        Ok(comments) => (StatusCode::OK, Json(json!(comments))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch comments",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn list_by_post(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    Valid(query): Valid<Query<V1PostCommentQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);

    println!("post_id {}", post_id);

    match PostComment::list_by_post(&state.db_pool, post_id, page).await {
        Ok((comments, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch comments for the post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn list_by_user(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    Valid(query): Valid<Query<V1PostCommentQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);

    match PostComment::list_by_user(&state.db_pool, user_id, page).await {
        Ok((comments, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch comments for the user",
            })),
        )
            .into_response(),
    }
}
