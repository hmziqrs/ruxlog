#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::schema;

#[derive(Queryable, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

impl User {
    pub async fn find_by_id(
        conn: &mut PgConnection,
        id: i32,
    ) -> Result<Self, diesel::result::Error> {
        use crate::db::schema::users::dsl::*;
        users.find(id).select(User::as_select()).first(conn)
    }

    pub async fn find_by_email(pool: Pool, email: &str) -> Result<Self, (StatusCode, String)> {
        use crate::db::schema::users::dsl::*;
        let conn = pool.get().await.map_err(internal_error)?;

        let result = conn
            .interact(|conn| {
                users
                    .filter(email.eq(email))
                    .select(User::as_select())
                    .first(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        return Ok(result);
    }

    pub async fn create_user(pool: Pool, new_user: NewUser) -> Result<Self, (StatusCode, String)> {
        use crate::db::schema::users::dsl::*;
        let conn = pool.get().await.map_err(internal_error)?;

        let res = conn
            .interact(|conn| {
                diesel::insert_into(schema::users::table)
                    .values(new_user)
                    .returning(User::as_returning())
                    .get_result(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        return Ok(res);
    }

    // pub fn create(conn: &mut PgConnection, name: &str, email: &str) -> Result<Self, diesel::result::Error> {
    //     use crate::db::schema::users::dsl::*;
    //     let new_user = NewUser {
    //         name: name.to_string(),
    //         email: email.to_string(),
    //     };
    //     diesel::insert_into(users).values(&new_user).get_result(conn)
    // }

    // pub fn update(conn: &PgConnection, id: i32, name: &str, email: &str) -> Result<Self, diesel::result::Error> {
    //     use crate::schema::users::dsl::*;
    //     diesel::update(users.find(id))
    //         .set((name.eq(name), email.eq(email)))
    //         .get_result(conn)
    // }

    // pub fn delete(conn: &PgConnection, id: i32) -> Result<usize, diesel::result::Error> {
    //     use crate::schema::users::dsl::*;
    //     diesel::delete(users.find(id)).execute(conn)
    // }
}
