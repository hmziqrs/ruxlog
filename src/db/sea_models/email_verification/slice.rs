use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewEmailVerification {
    pub user_id: i32,
    pub code: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct UpdateEmailVerification {
    pub code: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}