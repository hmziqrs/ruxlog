use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use axum_valid::Valid;
use serde_json::json;

use crate::{db::models::category::Category, services::auth::AuthSession, AppState};

use super::validator::{V1CategoryQueryParams, V1CreateCategoryPayload, V1UpdateCategoryPayload};

#[debug_handler]
pub async fn create(
    state: State<AppState>,
    payload: Valid<Json<V1CreateCategoryPayload>>,
) -> impl IntoResponse {
    let new_category = payload.into_inner().0.into_new_category();

    match Category::create(&state.db_pool, new_category).await {
        Ok(category) => (StatusCode::CREATED, Json(json!(category))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to create category",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn get_category_by_id(
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
) -> impl IntoResponse {
    match Category::get_category_by_id(&state.db_pool, category_id).await {
        Ok(Some(category)) => (StatusCode::OK, Json(json!(category))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Category not found" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch category",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn get_categories(
    State(state): State<AppState>,
    query: Valid<Query<V1CategoryQueryParams>>,
) -> impl IntoResponse {
    let parent_id = query.into_inner().0.parent_id;

    match Category::get_categories(&state.db_pool, parent_id).await {
        Ok(categories) => (StatusCode::OK, Json(json!(categories))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch categories",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
    payload: Valid<Json<V1UpdateCategoryPayload>>,
) -> impl IntoResponse {
    let update_category = payload.into_inner().0.into_update_category();

    match Category::update(&state.db_pool, category_id, update_category).await {
        Ok(Some(category)) => (StatusCode::OK, Json(json!(category))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Category does not exist",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to update category",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
) -> impl IntoResponse {
    match Category::delete(&state.db_pool, category_id).await {
        Ok(1) => (
            StatusCode::OK,
            Json(json!({ "message": "Category deleted successfully" })),
        )
            .into_response(),
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Category does not exist",
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "unexpected result",
                "message": "Internal server error occurred while deleting category",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to delete category",
            })),
        )
            .into_response(),
    }
}
