use super::UserRole;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

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
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Deserialize, Debug)]
pub struct ChangePasswordUser {
    pub password: String,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Deserialize, Debug)]
pub struct VerifiedUser {
    pub is_verified: bool,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminUserQuery {
    pub page: Option<u64>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<bool>,
    pub sorts: Option<Vec<crate::utils::SortParam>>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
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
    pub updated_at: DateTimeWithTimeZone,
}
