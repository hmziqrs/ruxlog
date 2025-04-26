use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use crate::{db::sea_models::tag::Entity as Tag, services::auth::AuthSession, AppState};

use super::validator::{V1CreateTagPayload, V1TagQueryParams, V1UpdateTagPayload};

/// Create a new tag using SeaORM
#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag creation requires authentication
    payload: Valid<Json<V1CreateTagPayload>>,
) -> impl IntoResponse {
    let new_tag = payload.into_inner().0.into_new_tag();

    match Tag::create(&state.sea_db, new_tag).await {
        Ok(tag) => (StatusCode::CREATED, Json(json!(tag))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to create tag",
            })),
        )
            .into_response(),
    }
}

/// Update an existing tag using SeaORM
#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    _auth: AuthSession, // Assuming tag update requires authentication
    Path(tag_id): Path<i32>,
    payload: Valid<Json<V1UpdateTagPayload>>,
) -> impl IntoResponse {
    let update_tag = payload.into_inner().0.into_update_tag();

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
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to update tag",
            })),
        )
            .into_response(),
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
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to delete tag",
            })),
        )
            .into_response(),
    }
}

/// Find a tag by ID using SeaORM
#[debug_handler]
pub async fn find_by_id(
    State(state): State<AppState>,
    Path(tag_id): Path<i32>,
) -> impl IntoResponse {
    match Tag::get_by_id(&state.sea_db, tag_id).await {
        Ok(Some(tag)) => (StatusCode::OK, Json(json!(tag))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Tag not found" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch tag",
            })),
        )
            .into_response(),
    }
}

/// Find all tags using SeaORM
#[debug_handler]
pub async fn find_all(State(state): State<AppState>) -> impl IntoResponse {
    match Tag::find_all(&state.sea_db).await {
        Ok(tags) => (StatusCode::OK, Json(json!(tags))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch tags",
            })),
        )
            .into_response(),
    }
}

/// Find tags with query using SeaORM
#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    query: Valid<Query<V1TagQueryParams>>,
) -> impl IntoResponse {
    let tag_query = query.into_inner().0.into_tag_query();
    let page = tag_query.page_no;

    match Tag::find_with_query(&state.sea_db, tag_query).await {
        Ok(tags) => (
            StatusCode::OK,
            Json(json!({
                "data": tags,
                "page": page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch tags",
            })),
        )
            .into_response(),
    }
}
