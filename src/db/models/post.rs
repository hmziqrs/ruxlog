#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use diesel::query_dsl::methods::FindDsl;
use diesel::QueryDsl;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Clone, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub published_at: Option<NaiveDateTime>,
    pub is_published: bool,
    pub slug: String,
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub category_id: Option<i32>,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::posts)]
pub struct NewPost {
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub published_at: Option<NaiveDateTime>,
    pub is_published: bool,
    pub slug: String,
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub category_id: Option<i32>,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::posts)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
    pub author_id: Option<i32>,
    pub published_at: Option<Option<NaiveDateTime>>,
    pub updated_at: NaiveDateTime,
    pub is_published: Option<bool>,
    pub slug: Option<String>,
    pub excerpt: Option<Option<String>>,
    pub featured_image_url: Option<Option<String>>,
    pub category_id: Option<Option<i32>>,
    pub view_count: Option<i32>,
    pub likes_count: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PostSortBy {
    Title,
    UpdatedAt,
    PublishedAt,
    ViewCount,
    LikesCount,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PostQuery {
    pub page_no: Option<i64>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<PostSortBy>,
    pub sort_order: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
}

const PER_PAGE: i64 = 12;

impl Post {
    pub async fn find_by_id(pool: &Pool, post_id: i32) -> Result<Option<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            posts
                .filter(id.eq(post_id))
                .first::<Post>(conn)
                .optional()
                .map_err(Into::into)
        })
        .await
    }

    pub async fn find_by_slug(pool: &Pool, post_slug: String) -> Result<Option<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            posts
                .filter(slug.eq(post_slug))
                .first::<Post>(conn)
                .optional()
                .map_err(Into::into)
        })
        .await
    }

    pub async fn find_all(pool: &Pool) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            posts.load::<Post>(conn).map_err(Into::into)
        })
        .await
    }

    pub async fn find_posts_with_query(
        pool: &Pool,
        query: PostQuery,
    ) -> Result<(Vec<Self>), DBError> {
        use crate::db::schema::posts::dsl::*;
        use diesel::dsl::count_star;
        use diesel::dsl::{count, sql};
        use diesel::sql_types::Bool;

        println!("{:?}", query.tag_ids);

        execute_db_operation(
            pool,
            move |conn| -> Result<Vec<Post>, diesel::result::Error> {
                let mut query_builder = posts.into_boxed();

                // Apply filters
                if let Some(author_id_filter) = query.author_id {
                    query_builder = query_builder.filter(author_id.eq(author_id_filter));
                }
                if let Some(category_id_filter) = query.category_id {
                    query_builder = query_builder.filter(category_id.eq(category_id_filter));
                }
                if let Some(is_published_filter) = query.is_published {
                    query_builder = query_builder.filter(is_published.eq(is_published_filter));
                }
                if let Some(search_term) = query.search {
                    let search_pattern = format!("%{}%", search_term.to_lowercase());
                    query_builder = query_builder.filter(
                        title
                            .ilike(search_pattern.clone())
                            .or(content.ilike(search_pattern)),
                    );
                }

                if let Some(tag_ids_filter) = query.tag_ids {
                    if !tag_ids_filter.is_empty() {
                        query_builder = query_builder.filter(tag_ids.overlaps_with(tag_ids_filter));
                    }
                }

                query_builder = match query.sort_by {
                    Some(PostSortBy::Title) => query_builder.order(title.asc()),
                    Some(PostSortBy::UpdatedAt) => query_builder.order(updated_at.desc()),
                    Some(PostSortBy::PublishedAt) => {
                        query_builder.order(published_at.desc().nulls_last())
                    }
                    Some(PostSortBy::ViewCount) => query_builder.order(view_count.desc()),
                    Some(PostSortBy::LikesCount) => query_builder.order(likes_count.desc()),
                    None => query_builder.order(created_at.desc()),
                };

                // Apply secondary sorting for consistency
                query_builder = match query.sort_order.as_deref() {
                    Some("asc") => query_builder.then_order_by(id.asc()),
                    _ => query_builder.then_order_by(id.desc()),
                };

                query_builder = query_builder
                    .limit(PER_PAGE)
                    .offset((query.page_no.unwrap_or(1) - 1) * PER_PAGE);

                // Execute query
                let items = query_builder.load::<Post>(conn)?;

                return Ok(items);
            },
        )
        .await
    }

    pub async fn find_paginated(pool: &Pool, page: i64) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = posts.count().get_result(conn)?;
            let items = posts
                .order(created_at.desc())
                .limit(PER_PAGE)
                .offset((page - 1) * PER_PAGE)
                .load::<Post>(conn)?;
            Ok((items, total))
        })
        .await
    }

    pub async fn find_published_paginated(
        pool: &Pool,
        page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = posts
                .filter(is_published.eq(true))
                .count()
                .get_result(conn)?;
            let items = posts
                .filter(is_published.eq(true))
                .order(updated_at.desc())
                // .order(published_at.desc())
                .limit(PER_PAGE)
                .offset((page - 1) * PER_PAGE)
                .load::<Post>(conn)?;
            Ok((items, total))
        })
        .await
    }

    pub async fn search_paginated(
        pool: &Pool,
        search_term: &str,
        page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;
        use diesel::expression_methods::ExpressionMethods;

        let search_pattern = format!("%{}%", search_term.to_lowercase());

        execute_db_operation(pool, move |conn| {
            let total = posts
                .filter(
                    title
                        .ilike(search_pattern.clone())
                        .or(content.ilike(search_pattern.clone())),
                )
                .count()
                .get_result(conn)?;

            let items = posts
                .filter(
                    title
                        .ilike(search_pattern.clone())
                        .or(content.ilike(search_pattern.clone())),
                )
                .order(created_at.desc())
                .limit(PER_PAGE)
                .offset((page - 1) * PER_PAGE)
                .load::<Post>(conn)?;

            Ok((items, total))
        })
        .await
    }

    pub async fn create(pool: &Pool, new_post: NewPost) -> Result<Self, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(posts)
                .values(&new_post)
                .get_result(conn)
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        post_id: i32,
        filter_user_id: i32,
        update_post: UpdatePost,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(posts.filter(id.eq(post_id).and(author_id.eq(filter_user_id))))
                .set(&update_post)
                .returning(Self::as_returning())
                .get_result(conn)
                .optional()
        })
        .await
    }

    pub async fn delete(pool: &Pool, filter_user_id: i32, post_id: i32) -> Result<usize, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(posts.filter(id.eq(post_id).and(author_id.eq(filter_user_id))))
                .execute(conn)
        })
        .await
    }
}
