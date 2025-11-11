#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use rand::{ Rng};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Identifiable, Selectable, Debug, Clone, Serialize, PartialEq)]
#[diesel(table_name = schema::categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub logo_image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::categories)]
pub struct NewCategory {
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub logo_image: Option<String>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::categories)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub parent_id: Option<Option<i32>>,
    pub description: Option<Option<String>>,
    pub cover_image: Option<Option<String>>,
    pub logo_image: Option<Option<String>>,
    pub updated_at: NaiveDateTime,
}

impl Category {
    pub async fn create(pool: &Pool, new_category: NewCategory) -> Result<Self, DBError> {
        use crate::db::schema::categories::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(categories)
                .values(&new_category)
                .returning(Self::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub async fn get_category_by_id(
        pool: &Pool,
        category_id: i32,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::categories::dsl::*;

        execute_db_operation(pool, move |conn| {
            categories
                .filter(id.eq(category_id))
                .first::<Category>(conn)
                .optional()
        })
        .await
    }

    pub async fn get_categories(
        pool: &Pool,
        parent_category_id: Option<i32>,
    ) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::categories::dsl::*;

        execute_db_operation(pool, move |conn| {
            let mut query = categories.into_boxed();

            if let Some(parent_id_filter) = parent_category_id {
                query = query.filter(parent_id.eq(parent_id_filter));
            }

            query.order(name.asc()).load::<Category>(conn)
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        category_id: i32,
        update_category: UpdateCategory,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::categories::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(categories.filter(id.eq(category_id)))
                .set(&update_category)
                .returning(Self::as_returning())
                .get_result(conn)
                .optional()
        })
        .await
    }

    pub async fn delete(pool: &Pool, category_id: i32) -> Result<usize, DBError> {
        use crate::db::schema::categories::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(categories.filter(id.eq(category_id))).execute(conn)
        })
        .await
    }

    pub async fn find_all(pool: &Pool) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::categories::dsl::*;
        execute_db_operation(pool, move |conn| categories.load::<Self>(conn)).await
    }
}
