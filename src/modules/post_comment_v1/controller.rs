use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::sea_models::post_comment,
    error::{ErrorCode, ErrorResponse},
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
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let new_comment = payload.0.into_new_post_comment(user.id);

    match post_comment::Entity::create(&state.sea_db, new_comment).await {
        Ok(comment) => Ok((StatusCode::CREATED, Json(json!(comment)))),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to create comment")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
    payload: ValidatedJson<V1UpdatePostCommentPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let update_comment = payload.0.into_update_post_comment();

    match post_comment::Entity::update(&state.sea_db, comment_id, user.id, update_comment).await {
        Ok(Some(comment)) => Ok((StatusCode::OK, Json(json!(comment)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Comment does not exist")),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to update comment")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    match post_comment::Entity::delete(&state.sea_db, comment_id, user.id).await {
        Ok(1) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Comment deleted successfully" })),
        )),
        Ok(0) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
            .with_message("Comment does not exist")),
        Ok(_) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Internal server error occurred while deleting comment")),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to delete comment")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn list(
    State(state): State<AppState>,
    query: ValidatedQuery<V1PostCommentQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let post_comment_query = query.0.into_post_comment_query();
    let page = post_comment_query.page_no;

    match post_comment::Entity::get_comments(&state.sea_db, post_comment_query).await {
        Ok((comments, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page,
            })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to fetch comments")
            .with_details(err.to_string())),
    }
}

#[debug_handler]
pub async fn list_by_post(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    query: ValidatedQuery<V1PostCommentQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let parsed_query = query.0.into_post_comment_query();
    let page = parsed_query.page_no;
    
    match post_comment::Entity::get_comments(
        &state.sea_db,
        post_comment::CommentQuery {
            post_id: Some(post_id),
            ..parsed_query
        },
    )
    .await
    {
        Ok((comments, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page,
            })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Failed to fetch comments for the post")
            .with_details(err.to_string())),
    }
}

