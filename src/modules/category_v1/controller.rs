use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::sea_models::category::Entity as Category,
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

use super::validator::{V1CategoryQueryParams, V1CreateCategoryPayload, V1UpdateCategoryPayload};

/// Create a new category using SeaORM
#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1CreateCategoryPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let new_category = payload.0.into_new_category();

    match Category::create(&state.sea_db, new_category).await {
        Ok(result) => Ok((StatusCode::CREATED, Json(json!(result)))),
        Err(err) => Err(err.into()),
    }
}

/// Update an existing category using SeaORM
#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(category_id): Path<i32>,
    payload: ValidatedJson<V1UpdateCategoryPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let update_category = payload.0.into_update_category();

    match Category::update(&state.sea_db, category_id, update_category).await {
        Ok(Some(category)) => Ok((StatusCode::OK, Json(json!(category)))),
        Ok(None) => {
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Category does not exist"))
        }
        Err(err) => Err(err.into()),
    }
}

/// Delete a category using SeaORM
#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(category_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match Category::delete(&state.sea_db, category_id).await {
        Ok(1) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Category deleted successfully" })),
        )),
        Ok(0) => {
            Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message("Category does not exist"))
        }
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Category deleted successfully" })),
        )),
        Err(err) => Err(err.into()),
    }
}

/// Find a category by ID using SeaORM
#[debug_handler]
pub async fn find_by_id_or_slug(
    State(state): State<AppState>,
    Path(slug_or_id): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let mut id: Option<i32> = None;
    let mut slug: Option<String> = None;

    match slug_or_id.parse::<i32>() {
        Ok(parsed_id) => {
            id = Some(parsed_id);
        }
        Err(_) => {
            slug = Some(slug_or_id);
        }
    }

    match Category::find_by_id_or_slug(&state.sea_db, id, slug).await {
        Ok(Some(category)) => Ok((StatusCode::OK, Json(json!(category)))),
        Ok(None) => {
            Err(ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Category not found"))
        }
        Err(err) => Err(err.into()),
    }
}

/// Find all categories using SeaORM
#[debug_handler]
pub async fn find_all(State(state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    match Category::find_all(&state.sea_db).await {
        Ok(categories) => Ok((StatusCode::OK, Json(json!(categories)))),
        Err(err) => Err(err.into()),
    }
}

/// Find categories with query using SeaORM
#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1CategoryQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let category_query = payload.0.into_category_query();
    let page = category_query.page_no.unwrap_or(1);

    match Category::find_with_query(&state.sea_db, category_query).await {
        Ok((categories, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": categories,
                "total": total,
                "per_page": Category::PER_PAGE,
                "page": page,
            })),
        )),
        Err(err) => Err(err.into()),
    }
}
