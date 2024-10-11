#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
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
    pub async fn find_by_email(pool: &Pool, user_email: String) -> Result<Option<Self>, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            users.filter(email.eq(user_email)).first(conn).optional()
        })
        .await
    }

    pub async fn create(pool: &Pool, new_user: NewUser) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(schema::users::table)
                .values(new_user)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
    }
}
