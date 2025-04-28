use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use super::UserRole;

#[derive(Deserialize, Debug)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Deserialize, Debug)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct ChangePasswordUser {
    pub password: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct VerifiedUser {
    pub is_verified: bool,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminUserQuery {
    pub page_no: Option<u64>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AdminCreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub avatar: Option<String>,
    pub is_verified: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct AdminUpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub avatar: Option<String>,
    pub is_verified: Option<bool>,
    pub updated_at: NaiveDateTime,
}