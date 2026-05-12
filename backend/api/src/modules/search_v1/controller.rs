//! Search controller — full-text search across published posts.

use axum::{extract::State, Json};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use validator::Validate;

use crate::db::sea_models::post;
use crate::error::codes::ErrorCode;
use crate::error::response::ErrorResponse;
use crate::AppState;

use super::validator::{SearchMeta, SearchQuery, SearchResponse, SearchResult};

#[derive(Debug, FromQueryResult, Deserialize)]
struct SearchRow {
    id: i32,
    title: String,
    slug: String,
    excerpt: Option<String>,
    status: String,
    published_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    created_at: chrono::DateTime<chrono::FixedOffset>,
    rank: Option<f64>,
}

pub async fn search(
    State(state): State<AppState>,
    Json(query): Json<SearchQuery>,
) -> Result<Json<SearchResponse>, ErrorResponse> {
    if query.validate().is_err() {
        return Err(ErrorResponse::new(ErrorCode::InvalidInput)
            .with_message("Search query must be 1-200 characters"));
    }

    let rows: Vec<SearchRow> = post::Entity::find()
        .filter(post::Column::Status.eq("published"))
        .filter(
            post::Column::Title
                .contains(&query.q)
                .or(post::Column::Excerpt.contains(&query.q))
                .or(post::Column::Slug.contains(&query.q)),
        )
        .order_by_desc(post::Column::CreatedAt)
        .offset(query.offset())
        .limit(query.per_page())
        .into_model::<SearchRow>()
        .all(&state.sea_db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    let total = rows.len() as u64;

    let results = rows
        .into_iter()
        .map(|r| SearchResult {
            id: r.id,
            title: r.title,
            slug: r.slug,
            excerpt: r.excerpt,
            status: r.status,
            published_at: r.published_at,
            created_at: r.created_at,
            rank: r.rank.unwrap_or(0.0),
        })
        .collect();

    Ok(Json(SearchResponse {
        data: results,
        meta: SearchMeta {
            total,
            page: query.page(),
            per_page: query.per_page(),
            query: query.q,
        },
    }))
}
