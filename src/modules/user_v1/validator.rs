use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::models::user::{AdminUserQuery, NewUser, UpdateUser};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateProfilePayload {
    #[validate(length(min = 1))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1))]
    pub password: Option<String>,
}

impl V1UpdateProfilePayload {
    pub fn into_update_user(self) -> UpdateUser {
        UpdateUser {
            name: self.name,
            email: self.email,
            updated_at: NaiveDateTime::from_timestamp(0, 0),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateUser {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1))]
    pub role: String,
}

impl AdminCreateUser {
    pub fn into_new_user(self) -> NewUser {
        NewUser {
            name: self.name,
            email: self.email,
            password: self.password,
            role: self.role,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminUpdateUser {
    #[validate(length(min = 1))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub is_verified: Option<bool>,
    #[validate(length(min = 1))]
    pub role: Option<String>,
}

impl AdminUpdateUser {
    pub fn into_update_user(self) -> UpdateUser {
        UpdateUser {
            name: self.name,
            email: self.email,
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminChangePassword {
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1AdminUserQueryParams {
    pub page_no: Option<i64>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub status: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

impl V1AdminUserQueryParams {
    pub fn into_user_query(self) -> AdminUserQuery {
        AdminUserQuery {
            page_no: self.page_no,
            email: self.email,
            name: self.name,
            role: self.role,
            status: self.status,
            created_at: self.created_at,
            updated_at: self.updated_at,
            sort_by: self.sort_by,
            sort_order: self.sort_order,
        }
    }
}
