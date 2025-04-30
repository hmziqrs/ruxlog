use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use super::Entity;

#[derive(Deserialize, Debug)]
pub struct NewForgotPassword {
    pub user_id: i32,
    pub code: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct UpdateForgotPassword {
    pub code: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegenerateForgotPassword {
    pub user_id: i32,
    pub code: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminForgotPasswordQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub code: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

impl NewForgotPassword {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        NewForgotPassword {
            user_id,
            code: Entity::generate_code(),
            expires_at: now + Entity::EXPIRY_TIME,
        }
    }
}

impl UpdateForgotPassword {
    pub fn regenerate() -> Self {
        let now = Utc::now().naive_utc();
        UpdateForgotPassword {
            code: Some(Entity::generate_code()),
            expires_at: Some(now + Entity::EXPIRY_TIME),
            updated_at: now,
        }
    }
}

impl RegenerateForgotPassword {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        RegenerateForgotPassword {
            user_id,
            code: Entity::generate_code(),
            updated_at: now,
        }
    }

    pub fn from_new(new: &NewForgotPassword) -> Self {
        RegenerateForgotPassword {
            user_id: new.user_id,
            code: new.code.clone(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}