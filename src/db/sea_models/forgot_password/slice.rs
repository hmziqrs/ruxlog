use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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