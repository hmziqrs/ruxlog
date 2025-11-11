#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use diesel::query_dsl::methods::FindDsl;
use diesel::QueryDsl;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize)]
#[diesel(table_name = schema::post_comments)]
pub struct PostComment {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub content: String,
    pub likes_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::post_comments)]
pub struct NewPostComment {
    pub post_id: i32,
    pub user_id: i32,
    pub content: String,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::post_comments)]
pub struct UpdatePostComment {
    pub content: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PostCommentSortBy {
    CreatedAt,
    UpdatedAt,
    LikesCount,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PostCommentQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub post_id: Option<i32>,
    pub search: Option<String>,
    pub sort_by: Option<PostCommentSortBy>,
    pub sort_order: Option<String>,
}

const MAX_PER_PAGE: i64 = 24;

impl PostComment {
    pub async fn create(pool: &Pool, new_comment: NewPostComment) -> Result<Self, DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(post_comments)
                .values(&new_comment)
                .get_result(conn)
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        comment_id: i32,
        filter_user_id: i32,
        update_comment: UpdatePostComment,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(post_comments.filter(id.eq(comment_id).and(user_id.eq(filter_user_id))))
                .set(&update_comment)
                .returning(Self::as_returning())
                .get_result(conn)
                .optional()
        })
        .await
    }

    pub async fn delete(
        pool: &Pool,
        comment_id: i32,
        filter_user_id: i32,
    ) -> Result<usize, DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(post_comments.filter(id.eq(comment_id).and(user_id.eq(filter_user_id))))
                .execute(conn)
        })
        .await
    }

    pub async fn list_all(pool: &Pool) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| post_comments.load::<PostComment>(conn)).await
    }

    pub async fn list_paginated(pool: &Pool, page: i64) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = post_comments.count().get_result(conn)?;
            let items = post_comments
                .order(created_at.desc())
                .limit(MAX_PER_PAGE)
                .offset((page - 1) * MAX_PER_PAGE)
                .load::<PostComment>(conn)?;
            Ok((items, total))
        })
        .await
    }

    pub async fn list_with_query(
        pool: &Pool,
        query: PostCommentQuery,
    ) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            let mut query_builder = post_comments.into_boxed();

            if let Some(user_id_filter) = query.user_id {
                query_builder = query_builder.filter(user_id.eq(user_id_filter));
            }
            if let Some(post_id_filter) = query.post_id {
                query_builder = query_builder.filter(post_id.eq(post_id_filter));
            }
            if let Some(search_term) = query.search {
                let search_pattern = format!("%{}%", search_term.to_lowercase());
                query_builder = query_builder.filter(content.ilike(search_pattern));
            }

            query_builder = match query.sort_by {
                Some(PostCommentSortBy::CreatedAt) => query_builder.order(created_at.desc()),
                Some(PostCommentSortBy::UpdatedAt) => query_builder.order(updated_at.desc()),
                Some(PostCommentSortBy::LikesCount) => query_builder.order(likes_count.desc()),
                None => query_builder.order(created_at.desc()),
            };

            query_builder = match query.sort_order.as_deref() {
                Some("asc") => query_builder.then_order_by(id.asc()),
                _ => query_builder.then_order_by(id.desc()),
            };

            let page = query.page_no.unwrap_or(1);
            query_builder = query_builder
                .limit(MAX_PER_PAGE)
                .offset((page - 1) * MAX_PER_PAGE);

            let items = query_builder.load::<PostComment>(conn)?;

            Ok(items)
        })
        .await
    }

    pub async fn list_by_post(
        pool: &Pool,
        query_post_id: i32,
        page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = post_comments
                .filter(post_id.eq(query_post_id))
                .count()
                .get_result(conn)?;
            let items = post_comments
                .filter(post_id.eq(query_post_id))
                .order(created_at.desc())
                .limit(MAX_PER_PAGE)
                .offset((page - 1) * MAX_PER_PAGE)
                .load::<PostComment>(conn)?;
            Ok((items, total))
        })
        .await
    }

    pub async fn list_by_user(
        pool: &Pool,
        query_user_id: i32,
        page: i64,
    ) -> Result<(Vec<Self>, i64), DBError> {
        use crate::db::schema::post_comments::dsl::*;

        execute_db_operation(pool, move |conn| {
            let total = post_comments
                .filter(user_id.eq(query_user_id))
                .count()
                .get_result(conn)?;
            let items = post_comments
                .filter(user_id.eq(query_user_id))
                .order(created_at.desc())
                .limit(MAX_PER_PAGE)
                .offset((page - 1) * MAX_PER_PAGE)
                .load::<PostComment>(conn)?;
            Ok((items, total))
        })
        .await
    }
}
