#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use rand::{ distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    models::forgot_password,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Clone, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::forgot_password)]
pub struct ForgotPassword {
    pub id: i32,
    pub user_id: i32,
    pub code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::forgot_password)]
pub struct NewForgotPassword {
    pub user_id: i32,
    pub code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::forgot_password)]
pub struct RegenerateForgotPassword {
    pub user_id: i32,
    pub code: String,
    pub updated_at: NaiveDateTime,
}

impl NewForgotPassword {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        NewForgotPassword {
            user_id,
            code: ForgotPassword::generate_code(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl RegenerateForgotPassword {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        RegenerateForgotPassword {
            user_id,
            code: ForgotPassword::generate_code(),
            updated_at: now,
        }
    }

    pub fn from_new(new: &NewForgotPassword) -> Self {
        RegenerateForgotPassword {
            user_id: new.user_id,
            code: new.code.clone(),
            updated_at: new.updated_at,
        }
    }
}

impl ForgotPassword {
    const DELAY_TIME: Duration = Duration::minutes(1);
    const EXPIRY_TIME: Duration = Duration::hours(3);

    pub async fn generate(pool: &Pool, db_user_id: i32) -> Result<Self, DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        let new_verification = NewForgotPassword::new(db_user_id);

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(forgot_password)
                .values(&new_verification)
                .on_conflict(user_id)
                .do_update()
                .set(RegenerateForgotPassword::from_new(&new_verification))
                .returning(ForgotPassword::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub async fn find_by_user_id(pool: &Pool, db_user_id: i32) -> Result<Option<Self>, DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        execute_db_operation(pool, move |conn| {
            forgot_password
                .filter(user_id.eq(db_user_id))
                .first(conn)
                .optional()
        })
        .await
    }

    pub async fn find_by_code(pool: &Pool, code: &str) -> Result<Option<Self>, DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        execute_db_operation(pool, move |conn| {
            forgot_password.filter(code.eq(code)).first(conn).optional()
        })
        .await
    }

    pub async fn find_by_user_id_and_code(
        pool: &Pool,
        db_user_id: i32,
        code: &str,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        execute_db_operation(pool, move |conn| {
            forgot_password
                .filter(user_id.eq(db_user_id).and(code.eq(code)))
                .first(conn)
                .optional()
        })
        .await
    }

    pub async fn delete(&self, pool: &Pool) -> Result<(), DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        let verification_id = self.id;

        execute_db_operation(pool, move |conn| {
            diesel::delete(forgot_password.filter(id.eq(verification_id))).execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn delete_by_user_id(pool: &Pool, db_user_id: i32) -> Result<(), DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(forgot_password.filter(user_id.eq(db_user_id))).execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn delete_by_code(code: &str, pool: &Pool) -> Result<(), DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(forgot_password.filter(code.eq(code))).execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn delete_by_user_id_and_code(
        pool: &Pool,
        db_user_id: i32,
        code: &str,
    ) -> Result<(), DBError> {
        use crate::db::schema::forgot_password::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(forgot_password.filter(user_id.eq(db_user_id).and(code.eq(code))))
                .execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub fn delete_query(
        conn: &mut PgConnection,
        auth_user_id: i32,
    ) -> Result<(), diesel::result::Error> {
        use crate::db::schema::forgot_password::dsl::*;

        diesel::delete(forgot_password)
            .filter(user_id.eq(auth_user_id))
            .execute(conn)?;

        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().naive_utc() > self.updated_at + Self::EXPIRY_TIME
    }

    pub fn is_in_delay(&self) -> bool {
        Utc::now().naive_utc() < self.updated_at + Self::DELAY_TIME
    }

    pub fn generate_code() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_lowercase()
    }
}
