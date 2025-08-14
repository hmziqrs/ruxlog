use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::sea_models::{comment_flag, post_comment},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

use super::validator::{
    V1AdminCommentFlagListQuery, V1AdminPostCommentListQuery, V1CreatePostCommentPayload,
    V1FlagCommentPayload, V1PostCommentQueryParams, V1UpdatePostCommentPayload,
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
        Err(err) => Err(err.into()),
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
        Ok(None) => {
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Comment does not exist"))
        }
        Err(err) => Err(err.into()),
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
        Ok(0) => {
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Comment does not exist"))
        }
        Ok(_) => Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Internal server error occurred while deleting comment")),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn list(
    State(state): State<AppState>,
    query: ValidatedJson<V1PostCommentQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let post_comment_query = query.0.into_post_comment_query();
    let page = post_comment_query.page_no.unwrap_or(1);

    match post_comment::Entity::get_comments(&state.sea_db, post_comment_query).await {
        Ok((comments, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page,
            })),
        )),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn list_by_post(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    query: ValidatedJson<V1PostCommentQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let parsed_query = query.0.into_post_comment_query();
    let page = parsed_query.page_no.unwrap_or(1);

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
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_list(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1AdminPostCommentListQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let query = payload.0.into_post_comment_query();
    let page = query.page_no.unwrap_or(1);

    match post_comment::Entity::get_comments(&state.sea_db, query).await {
        Ok((comments, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page
            })),
        )),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_flagged(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1AdminPostCommentListQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Placeholder: flagged filtering to be added when moderation fields exist
    let mut query = payload.0.into_post_comment_query();
    query.min_flags = Some(query.min_flags.unwrap_or(1));
    let page = query.page_no.unwrap_or(1);

    match post_comment::Entity::get_comments(&state.sea_db, query).await {
        Ok((comments, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": comments,
                "total": total,
                "page": page
            })),
        )),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_hide(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_hide(&state.sea_db, comment_id).await {
        Ok(Some(_)) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Hidden successfully" })),
        )),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_unhide(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_unhide(&state.sea_db, comment_id).await {
        Ok(Some(_)) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Unhidden successfully" })),
        )),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_delete(&state.sea_db, comment_id).await {
        Ok(affected) if affected > 0 => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Deleted successfully" })),
        )),
        Ok(_) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_flags_clear(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_flags_clear(&state.sea_db, comment_id).await {
        Ok(Some(_)) | Ok(None) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Flags cleared successfully" })),
        )),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn flag(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
    payload: ValidatedJson<V1FlagCommentPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let p = payload.0;

    let new_flag = comment_flag::slice::NewCommentFlag {
        comment_id,
        user_id: user.id,
        reason: p.reason,
    };

    match comment_flag::Entity::create(&state.sea_db, new_flag).await {
        Ok(_flag) => {
            let count = comment_flag::Entity::sync_flags_count(&state.sea_db, comment_id)
                .await
                .unwrap_or(0);
            Ok(Json(
                json!({ "message": "Flag recorded", "flags_count": count }),
            ))
        }
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_flags_list(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1AdminCommentFlagListQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let q = comment_flag::slice::CommentFlagQuery {
        page_no: payload.page,
        comment_id: payload.comment_id,
        user_id: payload.user_id,
        search_term: payload.search.clone(),
        sort_by: payload.sort_by.clone(),
        sort_order: payload.sort_order.clone(),
    };

    match comment_flag::Entity::list(&state.sea_db, q).await {
        Ok((items, total)) => Ok(Json(json!({
            "data": items,
            "total": total,
            "page": payload.page.unwrap_or(1)
        }))),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn admin_flags_summary(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match comment_flag::Entity::summary_for_comment(&state.sea_db, comment_id).await {
        Ok(summary) => Ok(Json(json!(summary))),
        Err(err) => Err(err.into()),
    }
}
