#![allow(unused)]
#![allow(clippy::all)]

use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Clone, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::email_verifications)]
pub struct EmailVerification {
    pub id: i32,
    pub user_id: i32,
    pub code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::email_verifications)]
pub struct NewEmailVerification {
    pub user_id: i32,
    pub code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::email_verifications)]
pub struct RegenerateEmailVerification {
    pub user_id: i32,
    pub code: String,
    pub updated_at: NaiveDateTime,
}

impl NewEmailVerification {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        NewEmailVerification {
            user_id,
            code: EmailVerification::generate_code(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl RegenerateEmailVerification {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        RegenerateEmailVerification {
            user_id,
            code: EmailVerification::generate_code(),
            updated_at: now,
        }
    }
}

impl EmailVerification {
    const DELAY_TIME: Duration = Duration::minutes(1);
    const EXPIRY_TIME: Duration = Duration::hours(3);

    pub fn create_query(
        conn: &mut PgConnection,
        auth_user_id: i32,
    ) -> Result<Self, diesel::result::Error> {
        use crate::db::schema::email_verifications::dsl::*;
        let new_verification = NewEmailVerification::new(auth_user_id);

        let email = diesel::insert_into(email_verifications)
            .values(&new_verification)
            .returning(EmailVerification::as_returning())
            .get_result(conn)?;

        Ok(email)
    }

    pub async fn regenerate(pool: &Pool, db_user_id: i32) -> Result<Self, DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        let updated_verification = RegenerateEmailVerification::new(db_user_id);

        execute_db_operation(pool, move |conn| {
            diesel::update(email_verifications.filter(user_id.eq(db_user_id)))
                .set(&updated_verification)
                .returning(EmailVerification::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub async fn find_by_user_id(pool: &Pool, db_user_id: i32) -> Result<Self, DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        execute_db_operation(pool, move |conn| {
            email_verifications
                .filter(user_id.eq(db_user_id))
                .first(conn)
        })
        .await
    }

    pub async fn find_by_code(pool: &Pool, code: &str) -> Result<Self, DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        execute_db_operation(pool, move |conn| {
            email_verifications.filter(code.eq(code)).first(conn)
        })
        .await
    }

    pub async fn find_by_user_id_and_code(
        pool: &Pool,
        db_user_id: i32,
        code: &str,
    ) -> Result<Self, DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        execute_db_operation(pool, move |conn| {
            email_verifications
                .filter(user_id.eq(db_user_id).and(code.eq(code)))
                .first(conn)
        })
        .await
    }

    pub async fn delete(&self, pool: &Pool) -> Result<(), DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        let verification_id = self.id;

        execute_db_operation(pool, move |conn| {
            diesel::delete(email_verifications.filter(id.eq(verification_id))).execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn delete_by_user_id(pool: &Pool, db_user_id: i32) -> Result<(), DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(email_verifications.filter(user_id.eq(db_user_id))).execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn delete_by_code(code: &str, pool: &Pool) -> Result<(), DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(email_verifications.filter(code.eq(code))).execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn delete_by_user_id_and_code(
        pool: &Pool,
        db_user_id: i32,
        code: &str,
    ) -> Result<(), DBError> {
        use crate::db::schema::email_verifications::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(email_verifications.filter(user_id.eq(db_user_id).and(code.eq(code))))
                .execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().naive_utc() > self.updated_at + Self::EXPIRY_TIME
    }

    pub fn is_in_delay(&self) -> bool {
        Utc::now().naive_utc() < self.updated_at + Self::DELAY_TIME
    }

    pub fn generate_code() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_lowercase()
    }

    pub async fn send_email(&self, email: &str) -> Result<(), DBError> {
        // Implement your email sending logic here
        // For example, using an email sending crate like lettre
        Ok(())
    }
}
