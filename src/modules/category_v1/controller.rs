use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use sea_orm::EntityTrait;
use serde_json::json;

use crate::{
    db::sea_models::category::Entity as Category,
    extractors::{ValidatedJson, ValidatedQuery},
    services::auth::AuthSession,
    AppState,
};

use super::validator::{V1CreateCategoryPayload, V1CategoryQueryParams, V1UpdateCategoryPayload};

/// Create a new category using SeaORM
#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1CreateCategoryPayload>,
) -> impl IntoResponse {
    let new_category = payload.0.into_new_category();

    Category::create(&state.sea_db, new_category).await
        .map(|result| (StatusCode::CREATED, Json(json!(result))))
        .map_err(IntoResponse::into_response)
}

/// Update an existing category using SeaORM
#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(category_id): Path<i32>,
    payload: ValidatedJson<V1UpdateCategoryPayload>,
) -> impl IntoResponse {
    let update_category = payload.0.into_update_category();

    match Category::update(&state.sea_db, category_id, update_category).await {
        Ok(Some(category)) => (StatusCode::OK, Json(json!(category))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Category does not exist",
            })),
        ).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Delete a category using SeaORM
#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(category_id): Path<i32>,
) -> impl IntoResponse {
    match Category::delete(&state.sea_db, category_id).await {
        Ok(1) => (
            StatusCode::OK,
            Json(json!({ "message": "Category deleted successfully" })),
        ).into_response(),
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Category does not exist",
            })),
        ).into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Category deleted successfully" })),
        ).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Find a category by ID using SeaORM
#[debug_handler]
pub async fn find_by_id_or_slug(
    State(state): State<AppState>,
    Path(slug_or_id): Path<String>,
) -> impl IntoResponse {
    let mut id: Option<i32> = None;
    let mut slug: Option<String> = None;
    
    match slug_or_id.parse::<i32>() {
        Ok(parsed_id) => {
            id = Some(parsed_id);
        },
        Err(_) => {
            slug = Some(slug_or_id);
        },
    }

    Category::find_by_id_or_slug(&state.sea_db, id, slug).await
        .map(|category| (StatusCode::OK, Json(json!(category))))
        .map_err(IntoResponse::into_response)
}

/// Find all categories using SeaORM
#[debug_handler]
pub async fn find_all(State(state): State<AppState>) -> impl IntoResponse {
    Category::find_all(&state.sea_db).await
        .map(|categories| (StatusCode::OK, Json(json!(categories))))
        .map_err(IntoResponse::into_response)
}

/// Find categories with query using SeaORM
#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    query: ValidatedQuery<V1CategoryQueryParams>,
) -> impl IntoResponse {
    let category_query = query.0.into_category_query();
    let page = category_query.page_no;

    Category::find_with_query(&state.sea_db, category_query).await
        .map(|(categories, total)| (
            StatusCode::OK,
            Json(json!({
                "total": total,
                "data": categories,
                "page": page,
            }))
        ))
        .map_err(IntoResponse::into_response)
}