#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
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
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PostQuery {
    pub pagination: Option<Pagination>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

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

    pub async fn find_all(pool: &Pool) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            posts.load::<Post>(conn).map_err(Into::into)
        })
        .await
    }

    pub async fn find_paginated(
        pool: &Pool,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = posts.count().get_result(conn)?;
            let items = posts
                .order(created_at.desc())
                .limit(per_page)
                .offset((page - 1) * per_page)
                .load::<Post>(conn)?;
            Ok((items, total))
        })
        .await
    }

    pub async fn find_published_paginated(
        pool: &Pool,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = posts
                .filter(is_published.eq(true))
                .count()
                .get_result(conn)?;
            let items = posts
                .filter(is_published.eq(true))
                .order(published_at.desc())
                .limit(per_page)
                .offset((page - 1) * per_page)
                .load::<Post>(conn)?;
            Ok((items, total))
        })
        .await
    }

    pub async fn search_paginated(
        pool: &Pool,
        search_term: &str,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;
        use diesel::pg::expression::dsl::any;

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
                .limit(per_page)
                .offset((page - 1) * per_page)
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
                .map_err(Into::into)
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        post_id: i32,
        update_post: UpdatePost,
    ) -> Result<Self, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(posts.filter(id.eq(post_id)))
                .set(&update_post)
                .get_result(conn)
                .map_err(Into::into)
        })
        .await
    }

    pub async fn delete(pool: &Pool, post_id: i32) -> Result<(), DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(posts.filter(id.eq(post_id)))
                .execute(conn)
                .map(|_| ())
                .map_err(Into::into)
        })
        .await
    }
}
