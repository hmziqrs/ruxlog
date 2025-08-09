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
    extractors::{ValidatedJson, ValidatedQuery},
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
) -> impl IntoResponse {
    let new_tag = payload.0.into_new_tag();

    Tag::create(&state.sea_db, new_tag)
        .await
        .map(|result| (StatusCode::CREATED, Json(json!(result))))
        .map_err(IntoResponse::into_response)
}

/// Update an existing tag using SeaORM
#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag update requires authentication
    Path(tag_id): Path<i32>,
    payload: ValidatedJson<V1UpdateTagPayload>,
) -> impl IntoResponse {
    let update_tag = payload.0.into_update_tag();

    match Tag::update(&state.sea_db, tag_id, update_tag).await {
        Ok(Some(tag)) => (StatusCode::OK, Json(json!(tag))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Tag does not exist",
            })),
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

/// Delete a tag using SeaORM
#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag deletion requires authentication
    Path(tag_id): Path<i32>,
) -> impl IntoResponse {
    match Tag::delete(&state.sea_db, tag_id).await {
        Ok(1) => (
            StatusCode::OK,
            Json(json!({ "message": "Tag deleted successfully" })),
        )
            .into_response(),
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Tag does not exist",
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Tag deleted successfully" })),
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

/// Find a tag by ID using SeaORM
#[debug_handler]
pub async fn find_by_id(
    State(state): State<AppState>,
    Path(tag_id): Path<i32>,
) -> impl IntoResponse {
    // Using our new find_by_id method with built-in not found handling
    Tag::find_by_id_with_404(&state.sea_db, tag_id)
        .await
        .map(|tag| (StatusCode::OK, Json(json!(tag))))
        .map_err(IntoResponse::into_response)
}

/// Find all tags using SeaORM
#[debug_handler]
pub async fn find_all(State(state): State<AppState>) -> impl IntoResponse {
    Tag::find_all(&state.sea_db)
        .await
        .map(|tags| (StatusCode::OK, Json(json!(tags))))
        .map_err(IntoResponse::into_response)
}

/// Find tags with query using SeaORM
#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1TagQueryParams>,
) -> impl IntoResponse {
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
        .map_err(IntoResponse::into_response)
}
