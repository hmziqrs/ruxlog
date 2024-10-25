use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_valid::Valid;

use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::post::Post, modules::post_v1::validator::V1UpdatePostPayload,
    services::auth::AuthSession, AppState,
};

use super::validator::{V1CreatePostPayload, V1PostQueryParams};

#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: Valid<Json<V1CreatePostPayload>>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let new_post = payload.into_inner().0.into_new_post(user.id);

    match Post::create(&state.db_pool, new_post).await {
        Ok(post) => (StatusCode::CREATED, Json(json!(post))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to create post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn find_by_id_or_slug(
    State(state): State<AppState>,
    Path(slug_or_id): Path<String>,
) -> impl IntoResponse {
    let query = match slug_or_id.parse::<i32>() {
        Ok(id) => Post::find_by_id(&state.db_pool, id).await,
        Err(_) => Post::find_by_slug(&state.db_pool, slug_or_id).await,
    };
    match query {
        Ok(Some(post)) => (StatusCode::OK, Json(json!(post))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Post not found" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
    payload: Valid<Json<V1UpdatePostPayload>>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    let update_post = payload.into_inner().0.into_update_post(user.id);

    match Post::update(&state.db_pool, post_id, user.id, update_post).await {
        Ok(Some(post)) => (StatusCode::OK, Json(json!(post))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "requested failed",
                "message": "Post does not exist",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to update post",
            })),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(post_id): Path<i32>,
) -> impl IntoResponse {
    let user = auth.user.unwrap();
    match Post::delete(&state.db_pool, user.id, post_id).await {
        Ok(1) => (
            StatusCode::OK,
            Json(json!({ "message": "Post deleted successfully" })),
        )
            .into_response(),
        Ok(0) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "request failed",
                "message": "Post does not exist",
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "unexpected result",
                "message": "Internal server error occurred while deleting post",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to delete post",
            })),
        )
            .into_response(),
    }
}

// #[debug_handler]
// pub async fn find_all_posts(State(state): State<AppState>) -> impl IntoResponse {
//     match Post::find_all(&state.db_pool).await {
//         Ok(posts) => (StatusCode::OK, Json(json!(posts))).into_response(),
//         Err(err) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({
//                 "error": err.to_string(),
//                 "message": "Failed to fetch posts",
//             })),
//         )
//             .into_response(),
//     }
// }

#[debug_handler]
pub async fn find_posts_with_query(
    State(state): State<AppState>,
    query: Valid<Json<V1PostQueryParams>>,
) -> impl IntoResponse {
    let post_query = query.into_inner().0.into_post_query();

    match Post::find_posts_with_query(&state.db_pool, post_query).await {
        Ok(posts) => (StatusCode::OK, Json(json!(posts))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch posts",
            })),
        )
            .into_response(),
    }
}

// #[debug_handler]
// pub async fn find_paginated_posts(
//     State(state): State<AppState>,
//     Valid(query): Valid<Query<V1PostQueryParams>>,
// ) -> impl IntoResponse {
//     let page = query.page.unwrap_or(1);
//     let per_page = query.per_page.unwrap_or(10);

//     match Post::find_paginated(&state.db_pool, page, per_page).await {
//         Ok((posts, total)) => (
//             StatusCode::OK,
//             Json(json!({
//                 "data": posts,
//                 "total": total,
//                 "page": page,
//                 "per_page": per_page,
//             })),
//         )
//             .into_response(),
//         Err(err) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({
//                 "error": err.to_string(),
//                 "message": "Failed to fetch paginated posts",
//             })),
//         )
//             .into_response(),
//     }
// }

#[debug_handler]
pub async fn find_published_posts(
    State(state): State<AppState>,
    Valid(query): Valid<Query<V1PostQueryParams>>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);

    match Post::find_published_paginated(&state.db_pool, page).await {
        Ok((posts, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": posts,
                "total": total,
                "page": page,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": err.to_string(),
                "message": "Failed to fetch published posts",
            })),
        )
            .into_response(),
    }
}

// #[debug_handler]
// pub async fn search_posts(
//     State(state): State<AppState>,
//     Valid(query): Valid<Query<V1PostQueryParams>>,
// ) -> impl IntoResponse {
//     let page = query.page.unwrap_or(1);
//     let per_page = query.per_page.unwrap_or(10);
//     let search_term = query.search.unwrap_or_default();

//     match Post::search_paginated(&state.db_pool, &search_term, page, per_page).await {
//         Ok((posts, total)) => (
//             StatusCode::OK,
//             Json(json!({
//                 "data": posts,
//                 "total": total,
//                 "page": page,
//                 "per_page": per_page,
//             })),
//         )
//             .into_response(),
//         Err(err) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({
//                 "error": err.to_string(),
//                 "message": "Failed to search posts",
//             })),
//         )
//             .into_response(),
//     }
// }
