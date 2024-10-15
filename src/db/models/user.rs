#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Clone, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_verified: bool,
    pub avatar: Option<String>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

impl User {
    pub async fn find_by_id(pool: &Pool, user_id: i32) -> Result<Option<Self>, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            users.filter(id.eq(user_id)).first(conn).optional()
        })
        .await
    }

    pub async fn find_by_email(pool: &Pool, user_email: String) -> Result<Option<Self>, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            users.filter(email.eq(user_email)).first(conn).optional()
        })
        .await
    }

    pub async fn create(pool: &Pool, new_user: NewUser) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;
        let pass = new_user.password.clone();
        let hash = task::spawn_blocking(move || password_auth::generate_hash(pass))
            .await
            .map_err(|_| DBError::PasswordHashError);

        let new_user = NewUser {
            password: hash?,
            ..new_user
        };

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(schema::users::table)
                .values(new_user)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub async fn update(pool: &Pool, user_id: i32, payload: UpdateUser) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(users.filter(id.eq(user_id)))
                .set(&payload)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
    }
}
