#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{errors::DBError, schema, utils::combine_errors};

#[derive(Queryable, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub avatar: Option<String>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn find_by_email(pool: Pool, email: &str) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;
        let conn = pool.get().await?;

        let result = conn
            .interact(|conn| {
                users
                    .filter(email.eq(email))
                    .select(User::as_select())
                    .first(conn)
            })
            .await;

        let flatten = combine_errors(result)?;

        return Ok(flatten);
    }

    pub async fn create(pool: Pool, new_user: NewUser) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;
        let conn = pool.get().await?;

        let res = conn
            .interact(|conn| {
                diesel::insert_into(schema::users::table)
                    .values(new_user)
                    .returning(User::as_returning())
                    .get_result(conn)
            })
            .await;

        let flatten = combine_errors(res)?;

        return Ok(flatten);
    }
}
