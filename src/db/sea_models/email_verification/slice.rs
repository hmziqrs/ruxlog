use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use super::Entity;

#[derive(Deserialize, Debug)]
pub struct NewEmailVerification {
    pub user_id: i32,
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateEmailVerification {
    pub code: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegenerateEmailVerification {
    pub user_id: i32,
    pub code: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminEmailVerificationQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub code: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

impl NewEmailVerification {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        NewEmailVerification {
            user_id,
            code: Entity::generate_code(),
        }
    }
}

impl UpdateEmailVerification {
    pub fn regenerate() -> Self {
        let now = Utc::now().naive_utc();
        UpdateEmailVerification {
            code: Some(Entity::generate_code()),
            updated_at: now,
        }
    }
}

impl RegenerateEmailVerification {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().naive_utc();
        RegenerateEmailVerification {
            user_id,
            code: Entity::generate_code(),
            updated_at: now,
        }
    }

    pub fn from_new(new: &NewEmailVerification) -> Self {
        RegenerateEmailVerification {
            user_id: new.user_id,
            code: new.code.clone(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}