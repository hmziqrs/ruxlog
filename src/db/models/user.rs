#![allow(unused)]
#![allow(clippy::all)]

use std::str::FromStr;

use axum::{http::StatusCode, Json};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::{
    errors::DBError,
    models::{email_verification::EmailVerification, forgot_password::ForgotPassword},
    schema::{self},
    utils::{combine_errors, execute_db_operation},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Moderator,
    Author,
    User,
}

impl UserRole {
    // Method to convert UserRole to String
    pub fn to_string(&self) -> String {
        match self {
            UserRole::SuperAdmin => "super-admin".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Moderator => "moderator".to_string(),
            UserRole::Author => "author".to_string(),
            UserRole::User => "user".to_string(),
        }
    }
}

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "super-admin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "author" => Ok(UserRole::Author),
            "user" => Ok(UserRole::User),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::SuperAdmin => 4,
            UserRole::Admin => 3,
            UserRole::Moderator => 2,
            UserRole::Author => 1,
            UserRole::User => 0,
        }
    }
}

#[derive(Queryable, Clone, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub avatar: Option<String>,
    pub is_verified: bool,
    pub role: UserRole,
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
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ChangePasswordUser {
    pub password: String,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct VerifiedUser {
    is_verified: bool,
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

    pub async fn find_by_email_and_forgot_password(
        pool: &Pool,
        user_email: String,
        otp_code: String,
    ) -> Result<Option<(Self, ForgotPassword)>, DBError> {
        use crate::db::schema::forgot_password::dsl as fp;
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            users
                .inner_join(fp::forgot_password)
                .filter(email.eq(user_email))
                .filter(fp::code.eq(otp_code))
                .select((User::as_select(), ForgotPassword::as_select()))
                .first(conn)
                .optional()
        })
        .await
    }

    pub async fn create(pool: &Pool, new_user: NewUser) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;
        let pass = new_user.password.clone();
        let hash = task::spawn_blocking(move || password_auth::generate_hash(pass))
            .await
            .map_err(|_| DBError::PasswordHashError)?;

        let new_user = NewUser {
            password: hash,
            ..new_user
        };

        execute_db_operation(pool, move |conn| {
            conn.transaction(|conn| {
                let user = diesel::insert_into(schema::users::table)
                    .values(new_user)
                    .returning(User::as_returning())
                    .get_result(conn)?;
                EmailVerification::create_query(conn, user.id)?;

                Ok(user)
            })
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

    pub async fn change_password(
        pool: &Pool,
        db_user_id: i32,
        new_pasword: String,
    ) -> Result<(), DBError> {
        use crate::db::schema::users::dsl::*;

        let hash = task::spawn_blocking(move || password_auth::generate_hash(new_pasword))
            .await
            .map_err(|_| DBError::PasswordHashError)?;

        let payload = ChangePasswordUser { password: hash };

        execute_db_operation(pool, move |conn| {
            conn.transaction(|conn| {
                diesel::update(users.filter(id.eq(db_user_id)))
                    .set(&payload)
                    .returning(User::as_returning())
                    .get_result(conn)?;

                ForgotPassword::delete_query(conn, db_user_id)?;
                Ok(())
            })
        })
        .await
    }

    pub async fn verify(pool: &Pool, user_id: i32) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::update(users.filter(id.eq(user_id)))
                .set(VerifiedUser { is_verified: true })
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
    }
}
