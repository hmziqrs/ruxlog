use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;

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
pub async fn create(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag creation requires authentication
    payload: ValidatedJson<V1CreateTagPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let new_tag = payload.0.into_new_tag();

    Tag::create(&state.sea_db, new_tag)
        .await
        .map(|result| (StatusCode::CREATED, Json(json!(result))))
}

/// Update an existing tag using SeaORM
#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag update requires authentication
    Path(tag_id): Path<i32>,
    payload: ValidatedJson<V1UpdateTagPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let update_tag = payload.0.into_update_tag();

    match Tag::update(&state.sea_db, tag_id, update_tag).await {
        Ok(Some(tag)) => Ok((StatusCode::OK, Json(json!(tag)))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::TagNotFound)),
        Err(err) => Err(err),
    }
}

/// Delete a tag using SeaORM
#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag deletion requires authentication
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match Tag::delete(&state.sea_db, tag_id).await {
        Ok(1) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Tag deleted successfully" })),
        )),
        Ok(0) => Err(ErrorResponse::new(ErrorCode::TagNotFound)),
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Tag deleted successfully" })),
        )),
        Err(err) => Err(err),
    }
}

/// Find a tag by ID using SeaORM
#[debug_handler]
pub async fn find_by_id(
    State(state): State<AppState>,
    Path(tag_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    Tag::find_by_id_with_404(&state.sea_db, tag_id)
        .await
        .map(|tag| (StatusCode::OK, Json(json!(tag))))
}

/// Find all tags using SeaORM
#[debug_handler]
pub async fn find_all(State(state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    Tag::find_all(&state.sea_db)
        .await
        .map(|tags| (StatusCode::OK, Json(json!(tags))))
}

/// Find tags with query using SeaORM
#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1TagQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let tag_query = payload.0.into_query();
    let page = tag_query.page.clone().unwrap_or(1);

    Tag::find_with_query(&state.sea_db, tag_query)
        .await
        .map(|(tags, total)| {
            (
                StatusCode::OK,
                Json(json!({
                    "data": tags,
                    "total": total,
                    "per_page": Tag::PER_PAGE,
                    "page": page,
                })),
            )
        })
}
