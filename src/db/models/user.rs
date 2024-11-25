#![allow(unused)]
#![allow(clippy::all)]

use crate::db::{
    errors::DBError,
    models::{email_verification::EmailVerification, forgot_password::ForgotPassword},
    schema::{self},
    utils::{combine_errors, execute_db_operation},
};
use axum::{http::StatusCode, Json};
use chrono::NaiveDateTime;
use deadpool_diesel::postgres::Pool;
use diesel::{
    associations::HasTable, dsl::count_star, prelude::*, query_builder::QueryFragment,
    sql_types::BigInt,
};
use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, str::FromStr};
use tokio::task;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, diesel_derive_enum::DbEnum,
)]
#[ExistingTypePath = "crate::db::schema::sql_types::UserRole"]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    #[db_rename = "super-admin"]
    SuperAdmin,
    Admin,
    Moderator,
    Author,
    User,
}

impl UserRole {
    pub fn from_str(s: &str) -> Result<Self, String> {
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

impl UserRole {
    pub fn to_i32(&self) -> i32 {
        match self {
            UserRole::SuperAdmin => 4,
            UserRole::Admin => 3,
            UserRole::Moderator => 2,
            UserRole::Author => 1,
            UserRole::User => 0,
        }
    }

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

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        UserRole::from_str(s).unwrap_or(UserRole::User)
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
    // pub role: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct ChangePasswordUser {
    pub password: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct VerifiedUser {
    is_verified: bool,
    updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AdminUserQuery {
    pub page_no: Option<i64>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct AdminCreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub avatar: Option<String>,
    pub is_verified: Option<bool>,
}

#[derive(Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = schema::users)]
pub struct AdminUpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub avatar: Option<String>,
    pub is_verified: Option<bool>,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub const ADMIN_PER_PAGE: i64 = 20;
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

        let payload = ChangePasswordUser {
            password: hash,
            updated_at: chrono::Utc::now().naive_utc(),
        };

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
                .set(VerifiedUser {
                    is_verified: true,
                    updated_at: chrono::Utc::now().naive_utc(),
                })
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub fn get_role(&self) -> UserRole {
        self.role
        // UserRole::from_str(&self.role).unwrap()
    }

    pub fn is_user(&self) -> bool {
        self.get_role().to_i32() >= UserRole::User.to_i32()
    }

    pub fn is_author(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Author.to_i32()
    }

    pub fn is_mod(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Moderator.to_i32()
    }

    pub fn is_admin(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Admin.to_i32()
    }

    pub fn is_super_admin(&self) -> bool {
        self.get_role().to_i32() >= UserRole::SuperAdmin.to_i32()
    }

    pub async fn admin_create(pool: &Pool, new_user: AdminCreateUser) -> Result<Self, DBError> {
        use crate::db::schema::users::dsl::*;
        let pass = new_user.password.clone();
        let hash = task::spawn_blocking(move || password_auth::generate_hash(pass))
            .await
            .map_err(|_| DBError::PasswordHashError)?;

        let new_user = AdminCreateUser {
            password: hash,
            ..new_user
        };

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(users)
                .values(&new_user)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub async fn admin_delete(pool: &Pool, user_id: i32) -> Result<usize, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::delete(users.filter(id.eq(user_id))).execute(conn)
        })
        .await
    }

    pub async fn admin_update(
        pool: &Pool,
        user_id: i32,
        mut payload: AdminUpdateUser,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::users::dsl::*;

        if let Some(pswd) = payload.password {
            let hash = task::spawn_blocking(move || password_auth::generate_hash(pswd))
                .await
                .map_err(|_| DBError::PasswordHashError)?;

            payload.password = Some(hash);
        }

        execute_db_operation(pool, move |conn| {
            diesel::update(users.filter(id.eq(user_id)))
                .set(&payload)
                .returning(User::as_returning())
                .get_result(conn)
                .optional()
        })
        .await
    }

    pub async fn admin_change_password(
        pool: &Pool,
        user_id: i32,
        new_password: String,
    ) -> Result<(), DBError> {
        use crate::db::schema::users::dsl::*;

        let hash = task::spawn_blocking(move || password_auth::generate_hash(new_password))
            .await
            .map_err(|_| DBError::PasswordHashError)?;

        let payload = ChangePasswordUser {
            password: hash,
            updated_at: chrono::Utc::now().naive_utc(),
        };

        execute_db_operation(pool, move |conn| {
            diesel::update(users.filter(id.eq(user_id)))
                .set(&payload)
                .execute(conn)
        })
        .await
        .map(|_| ())
    }

    pub async fn admin_view(pool: &Pool, user_id: i32) -> Result<Option<Self>, DBError> {
        use crate::db::schema::users::dsl::*;

        execute_db_operation(pool, move |conn| {
            users.filter(id.eq(user_id)).get_result(conn).optional()
        })
        .await
    }

    pub async fn admin_list(pool: &Pool, query: AdminUserQuery) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::users::dsl::*;
        use diesel::dsl::sql;

        execute_db_operation(pool, move |conn| {
            let mut query_builder = users.into_boxed();

            if let Some(email_filter) = query.email {
                query_builder = query_builder.filter(email.ilike(format!("%{}%", email_filter)));
            }
            if let Some(name_filter) = query.name {
                query_builder = query_builder.filter(name.ilike(format!("%{}%", name_filter)));
            }
            if let Some(role_filter) = query.role {
                query_builder = query_builder.filter(role.eq(role_filter));
            }
            if let Some(status_filter) = query.status {
                query_builder = query_builder.filter(is_verified.eq(status_filter));
            }
            if let Some(created_at_filter) = query.created_at {
                query_builder = query_builder.filter(created_at.eq(created_at_filter));
            }
            if let Some(updated_at_filter) = query.updated_at {
                query_builder = query_builder.filter(updated_at.eq(updated_at_filter));
            }

            if let Some(sort_by_fields) = query.sort_by {
                for field in sort_by_fields {
                    query_builder = match field.as_str() {
                        "email" => query_builder.then_order_by(email.asc()),
                        "name" => query_builder.then_order_by(name.asc()),
                        "role" => query_builder.then_order_by(role.asc()),
                        "status" => query_builder.then_order_by(is_verified.asc()),
                        "created_at" => query_builder.then_order_by(created_at.asc()),
                        "updated_at" => query_builder.then_order_by(updated_at.asc()),
                        _ => query_builder,
                    };
                }
            }

            query_builder = match query.sort_order.as_deref() {
                Some("asc") => query_builder.then_order_by(id.asc()),
                _ => query_builder.then_order_by(id.desc()),
            };

            let page = query.page_no.unwrap_or(1);

            let items = query_builder
                .select(users::all_columns())
                .limit(Self::ADMIN_PER_PAGE)
                .offset((page - 1) * Self::ADMIN_PER_PAGE)
                .load::<User>(conn)?;

            Ok(items)
        })
        .await
    }
}
