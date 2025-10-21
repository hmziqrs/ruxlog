use super::Entity;
use chrono::Utc;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct UpdateEmailVerification {
    pub code: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegenerateEmailVerification {
    pub user_id: i32,
    pub code: String,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminEmailVerificationQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub code: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

impl UpdateEmailVerification {
    pub fn regenerate() -> Self {
        let now = Utc::now().fixed_offset();
        UpdateEmailVerification {
            code: Some(Entity::generate_code()),
            updated_at: now,
        }
    }
}

impl RegenerateEmailVerification {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now().fixed_offset();
        RegenerateEmailVerification {
            user_id,
            code: Entity::generate_code(),
            updated_at: now,
        }
    }
}
