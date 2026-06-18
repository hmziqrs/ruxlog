use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

/// Input for creating a forgot-password row. `code_hash` is the keyed hash of
/// the plaintext code (computed by the caller via `utils::code_hash::hash_code`),
/// never the plaintext itself.
#[derive(Deserialize, Debug)]
pub struct NewForgotPassword {
    pub user_id: i32,
    pub code_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminForgotPasswordQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub code_hash: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}
