use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info, instrument, warn};

use crate::{
    db::sea_models::{comment_flag, post_comment},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

use super::validator::{
    V1AdminCommentFlagListQuery, V1AdminPostCommentListQuery, V1CreatePostCommentPayload,
    V1FlagCommentPayload, V1UpdatePostCommentPayload,
};

#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id = auth.user.as_ref().map(|u| u.id), post_id, comment_id))]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<V1CreatePostCommentPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let new_comment = payload.0.into_new_post_comment(user.id);
    tracing::Span::current().record("post_id", new_comment.post_id);

    match post_comment::Entity::create(&state.sea_db, new_comment).await {
        Ok(comment) => {
            tracing::Span::current().record("comment_id", comment.id);
            info!(
                user_id = user.id,
                post_id = comment.post_id,
                comment_id = comment.id,
                "Comment created"
            );
            Ok((StatusCode::CREATED, Json(json!(comment))))
        }
        Err(err) => {
            error!(user_id = user.id, "Failed to create comment: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id = auth.user.as_ref().map(|u| u.id), comment_id))]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
    payload: ValidatedJson<V1UpdatePostCommentPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    let update_comment = payload.0.into_update_post_comment();

    match post_comment::Entity::update(&state.sea_db, comment_id, user.id, update_comment).await {
        Ok(Some(comment)) => {
            info!(user_id = user.id, comment_id, "Comment updated");
            Ok((StatusCode::OK, Json(json!(comment))))
        }
        Ok(None) => {
            warn!(
                user_id = user.id,
                comment_id, "Comment not found for update"
            );
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Comment does not exist"))
        }
        Err(err) => {
            error!(
                user_id = user.id,
                comment_id, "Failed to update comment: {}", err
            );
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth), fields(user_id = auth.user.as_ref().map(|u| u.id), comment_id))]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = auth.user.unwrap();
    match post_comment::Entity::delete(&state.sea_db, comment_id, user.id).await {
        Ok(1) => {
            info!(user_id = user.id, comment_id, "Comment deleted");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Comment deleted successfully" })),
            ))
        }
        Ok(0) => {
            warn!(
                user_id = user.id,
                comment_id, "Comment not found for delete"
            );
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Comment does not exist"))
        }
        Ok(_) => {
            info!(user_id = user.id, comment_id, "Comment deleted");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Comment deleted successfully" })),
            ))
        }
        Err(err) => {
            error!(
                user_id = user.id,
                comment_id, "Failed to delete comment: {}", err
            );
            Err(err.into())
        }
    }
}

/// Find comments by post ID (public use)
#[debug_handler]
#[instrument(skip(state), fields(post_id))]
pub async fn find_all_by_post(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match post_comment::Entity::find_all_by_post(&state.sea_db, post_id).await {
        Ok(comments) => {
            info!(
                post_id,
                count = comments.len(),
                "Comments retrieved for post"
            );
            Ok((StatusCode::OK, Json(json!(comments))))
        }
        Err(err) => {
            error!(post_id, "Failed to retrieve comments: {}", err);
            Err(err.into())
        }
    }
}

/// Find comments with query (dashboard use)
#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1AdminPostCommentListQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let comment_query = payload.0.into_post_comment_query();
    let page = comment_query.page_no.unwrap_or(1);

    match post_comment::Entity::find_with_query(&state.sea_db, comment_query).await {
        Ok((comments, total)) => {
            info!(total, page, "Admin listed comments");
            Ok((
                StatusCode::OK,
                Json(json!({
                    "data": comments,
                    "total": total,
                    "per_page": post_comment::Entity::PER_PAGE,
                    "page": page,
                })),
            ))
        }
        Err(err) => {
            error!("Failed to list comments: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth), fields(comment_id, admin_id = auth.user.as_ref().map(|u| u.id)))]
pub async fn admin_hide(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_hide(&state.sea_db, comment_id).await {
        Ok(Some(_)) => {
            info!(comment_id, "Admin hid comment");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Hidden successfully" })),
            ))
        }
        Ok(None) => {
            warn!(comment_id, "Comment not found for hide");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound))
        }
        Err(err) => {
            error!(comment_id, "Failed to hide comment: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth), fields(comment_id, admin_id = auth.user.as_ref().map(|u| u.id)))]
pub async fn admin_unhide(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_unhide(&state.sea_db, comment_id).await {
        Ok(Some(_)) => {
            info!(comment_id, "Admin unhid comment");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Unhidden successfully" })),
            ))
        }
        Ok(None) => {
            warn!(comment_id, "Comment not found for unhide");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound))
        }
        Err(err) => {
            error!(comment_id, "Failed to unhide comment: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth), fields(comment_id, admin_id = auth.user.as_ref().map(|u| u.id)))]
pub async fn admin_delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_delete(&state.sea_db, comment_id).await {
        Ok(affected) if affected > 0 => {
            info!(comment_id, "Admin deleted comment");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Deleted successfully" })),
            ))
        }
        Ok(_) => {
            warn!(comment_id, "Comment not found for admin delete");
            Err(ErrorResponse::new(ErrorCode::RecordNotFound))
        }
        Err(err) => {
            error!(comment_id, "Failed to admin delete comment: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth), fields(comment_id, admin_id = auth.user.as_ref().map(|u| u.id)))]
pub async fn admin_flags_clear(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let _user = auth.user.unwrap();

    match post_comment::Entity::admin_flags_clear(&state.sea_db, comment_id).await {
        Ok(Some(_)) | Ok(None) => {
            info!(comment_id, "Admin cleared comment flags");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Flags cleared successfully" })),
            ))
        }
        Err(err) => {
            error!(comment_id, "Failed to clear flags: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, auth, payload), fields(user_id = auth.user.as_ref().map(|u| u.id), comment_id))]
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
            info!(
                user_id = user.id,
                comment_id,
                flags_count = count,
                "Comment flagged"
            );
            Ok(Json(
                json!({ "message": "Flag recorded", "flags_count": count }),
            ))
        }
        Err(err) => {
            error!(
                user_id = user.id,
                comment_id, "Failed to flag comment: {}", err
            );
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth, payload))]
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
        Ok((items, total)) => {
            info!(
                total,
                page = payload.page.unwrap_or(1),
                "Admin listed comment flags"
            );
            Ok(Json(json!({
                "data": items,
                "total": total,
                "page": payload.page.unwrap_or(1)
            })))
        }
        Err(err) => {
            error!("Failed to list comment flags: {}", err);
            Err(err.into())
        }
    }
}

#[debug_handler]
#[instrument(skip(state, _auth), fields(comment_id))]
pub async fn admin_flags_summary(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(comment_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match comment_flag::Entity::summary_for_comment(&state.sea_db, comment_id).await {
        Ok(summary) => {
            info!(comment_id, "Admin viewed comment flags summary");
            Ok(Json(json!(summary)))
        }
        Err(err) => {
            error!(comment_id, "Failed to get flags summary: {}", err);
            Err(err.into())
        }
    }
}
