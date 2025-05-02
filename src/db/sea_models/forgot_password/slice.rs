use chrono::Utc;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use super::Entity;

#[derive(Deserialize, Debug)]
pub struct NewForgotPassword {
    pub user_id: i32,
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateForgotPassword {
    pub code: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegenerateForgotPassword {
    pub user_id: i32,
    pub code: String,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminForgotPasswordQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub code: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

impl NewForgotPassword {
    pub fn new(user_id: i32) -> Self {
        NewForgotPassword {
            user_id,
            code: Entity::generate_code(),
        }
    }
}

impl UpdateForgotPassword {
    pub fn regenerate() -> Self {
        let now = Utc::now().fixed_offset();
        UpdateForgotPassword {
            code: Some(Entity::generate_code()),
            updated_at: now,
        }
    }
}

impl RegenerateForgotPassword {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().fixed_offset();
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
            updated_at: Utc::now().fixed_offset(),
        }
    }
}