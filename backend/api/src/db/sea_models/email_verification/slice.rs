use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminEmailVerificationQuery {
    pub page_no: Option<i64>,
    pub user_id: Option<i32>,
    pub code_hash: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}
