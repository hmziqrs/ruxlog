use chrono::NaiveDateTime;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use diesel::QueryDsl;
use serde::{Deserialize, Serialize};

use crate::db::{errors::DBError, schema, utils::execute_db_operation};

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Clone)]
#[diesel(table_name = schema::tags)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::tags)]
pub struct NewTag {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::tags)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<Option<String>>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TagQuery {
    pub page_no: Option<i64>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
}

impl Tag {
    const PER_PAGE: i64 = 20;
    pub async fn create(pool: &Pool, new_tag: NewTag) -> Result<Self, DBError> {
        use crate::db::schema::tags::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(tags).values(&new_tag).get_result(conn)
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        tag_id: i32,
        update_tag: UpdateTag,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::tags::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(tags.filter(id.eq(tag_id)))
                .set(&update_tag)
                .returning(Self::as_returning())
                .get_result(conn)
                .optional()
        })
        .await
    }

    pub async fn delete(pool: &Pool, tag_id: i32) -> Result<usize, DBError> {
        use crate::db::schema::tags::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(tags.filter(id.eq(tag_id))).execute(conn)
        })
        .await
    }

    pub async fn find_by_id(pool: &Pool, tag_id: i32) -> Result<Option<Self>, DBError> {
        use crate::db::schema::tags::dsl::*;

        execute_db_operation(pool, move |conn| {
            tags.filter(id.eq(tag_id)).first::<Self>(conn).optional()
        })
        .await
    }

    pub async fn find_all(pool: &Pool) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::tags::dsl::*;

        execute_db_operation(pool, move |conn| tags.load::<Self>(conn)).await
    }

    pub async fn find_with_query(pool: &Pool, query: TagQuery) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::tags::dsl::*;

        execute_db_operation(pool, move |conn| {
            let mut query_builder = tags.into_boxed();

            if let Some(search_term) = query.search {
                let search_pattern = format!("%{}%", search_term.to_lowercase());
                query_builder = query_builder.filter(
                    name.ilike(search_pattern.clone())
                        .or(description.ilike(search_pattern)),
                );
            }

            query_builder = match query.sort_order.as_deref() {
                Some("asc") => query_builder.order(name.asc()),
                _ => query_builder.order(name.desc()),
            };

            let page = query.page_no.unwrap_or(1);
            let items = query_builder
                .limit(Self::PER_PAGE)
                .offset((page - 1) * Self::PER_PAGE)
                .load::<Tag>(conn)?;

            Ok(items)
        })
        .await
    }
}
