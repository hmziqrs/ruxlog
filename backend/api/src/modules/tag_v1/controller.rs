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
    db::sea_models::tag::Entity as Tag,
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

use super::validator::{V1CreateTagPayload, V1TagQueryParams, V1UpdateTagPayload};

/// Create a new tag using SeaORM
#[debug_handler]
#[instrument(skip(state, _auth, payload), fields(tag_id))]
pub async fn create(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag creation requires authentication
    payload: ValidatedJson<V1CreateTagPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let new_tag = payload.0.into_new_tag();

    match Tag::create(&state.sea_db, new_tag).await {
        Ok(result) => {
            tracing::Span::current().record("tag_id", result.id);
            info!(tag_id = result.id, "Tag created");
            Ok((StatusCode::CREATED, Json(json!(result))))
        }
        Err(err) => {
            error!("Failed to create tag: {}", err);
            Err(err)
        }
    }
}

/// Update an existing tag using SeaORM
#[debug_handler]
#[instrument(skip(state, _auth, payload), fields(tag_id))]
pub async fn update(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag update requires authentication
    Path(tag_id): Path<i32>,
    payload: ValidatedJson<V1UpdateTagPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let update_tag = payload.0.into_update_tag();

    match Tag::update(&state.sea_db, tag_id, update_tag).await {
        Ok(Some(tag)) => {
            info!(tag_id, "Tag updated");
            Ok((StatusCode::OK, Json(json!(tag))))
        }
        Ok(None) => {
            warn!(tag_id, "Tag not found for update");
            Err(ErrorResponse::new(ErrorCode::TagNotFound))
        }
        Err(err) => {
            error!(tag_id, "Failed to update tag: {}", err);
            Err(err)
        }
    }
}

/// Delete a tag using SeaORM
#[debug_handler]
#[instrument(skip(state, _auth), fields(tag_id))]
pub async fn delete(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag deletion requires authentication
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match Tag::delete(&state.sea_db, tag_id).await {
        Ok(1) => {
            info!(tag_id, "Tag deleted");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Tag deleted successfully" })),
            ))
        }
        Ok(0) => {
            warn!(tag_id, "Tag not found for delete");
            Err(ErrorResponse::new(ErrorCode::TagNotFound))
        }
        Ok(_) => {
            info!(tag_id, "Tag deleted");
            Ok((
                StatusCode::OK,
                Json(json!({ "message": "Tag deleted successfully" })),
            ))
        }
        Err(err) => {
            error!(tag_id, "Failed to delete tag: {}", err);
            Err(err)
        }
    }
}

/// Find a tag by ID using SeaORM
#[debug_handler]
#[instrument(skip(state), fields(tag_id))]
pub async fn find_by_id(
    State(state): State<AppState>,
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match Tag::find_by_id_with_404(&state.sea_db, tag_id).await {
        Ok(tag) => {
            info!(tag_id, "Tag retrieved");
            Ok((StatusCode::OK, Json(json!(tag))))
        }
        Err(err) => {
            warn!(tag_id, "Tag not found");
            Err(err)
        }
    }
}

/// Find all tags using SeaORM
#[debug_handler]
#[instrument(skip(state))]
pub async fn find_all(State(state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    match Tag::find_all(&state.sea_db).await {
        Ok(tags) => {
            info!(count = tags.len(), "All tags retrieved");
            Ok((StatusCode::OK, Json(json!(tags))))
        }
        Err(err) => {
            error!("Failed to retrieve all tags: {}", err);
            Err(err)
        }
    }
}

/// Find tags with query using SeaORM
#[debug_handler]
#[instrument(skip(state, payload))]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1TagQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let tag_query = payload.0.into_query();
    let page = tag_query.page.clone().unwrap_or(1);

    match Tag::find_with_query(&state.sea_db, tag_query).await {
        Ok((tags, total)) => {
            info!(total, page, "Tags retrieved with query");
            Ok((
                StatusCode::OK,
                Json(json!({
                    "data": tags,
                    "total": total,
                    "per_page": Tag::PER_PAGE,
                    "page": page,
                })),
            ))
        }
        Err(err) => {
            error!("Failed to query tags: {}", err);
            Err(err)
        }
    }
}
