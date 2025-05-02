use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use crate::{
    db::sea_models::post_comment,
    extractors::{ValidatedJson, ValidatedQuery},
    services::auth::AuthSession,
    AppState,
};

use super::validator::{
    V1CreatePostCommentPayload, V1PostCommentQueryParams, V1UpdatePostCommentPayload,
};

#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1CreatePostCommentPayload>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let new_comment = payload.0.into_new_post_comment(user.id);

    match post_comment::Entity::create(&state.sea_db, new_comment).await {
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
    payload: ValidatedJson<V1UpdatePostCommentPayload>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let update_comment = payload.0.into_update_post_comment();

    match post_comment::Entity::update(&state.sea_db, comment_id, user.id, update_comment).await {
        Ok(Some(comment)) => (StatusCode::OK, Json(json!(comment))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Comment does not exist",
            })),
        )
            .into_response(),
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
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    match post_comment::Entity::delete(&state.sea_db, comment_id, user.id).await {
        Ok(1) => (
            StatusCode::OK,
            Json(json!({ "message": "Comment deleted successfully" })),
        )
            .into_response(),
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Comment does not exist",
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "unexpected result",
                "message": "Internal server error occurred while deleting comment",
            })),
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
pub async fn list(
    State(state): State<AppState>,
    query: Valid<Query<V1PostCommentQueryParams>>,
) -> impl IntoResponse {
    let post_comment_query = query.into_inner().0.into_post_comment_query();

    match post_comment::Entity::search(&state.sea_db, post_comment_query).await {
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
    query: ValidatedQuery<V1PostCommentQueryParams>,
) -> impl IntoResponse {
    let parsed_query = query.0.into_post_comment_query();
    match post_comment::Entity::search(
        &state.sea_db,
        post_comment::CommentQuery {
            post_id: Some(post_id),
            ..parsed_query
        },
    )
    .await
    {
        Ok((comments, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": parsed_query.page_no,
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

