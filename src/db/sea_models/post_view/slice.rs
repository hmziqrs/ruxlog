use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewPostView {
    pub post_id: i32,
    pub ip_address: String,
    pub user_agent: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostViewQuery {
    pub page_no: Option<u64>,
    pub post_id: i32,
    pub ip_address: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}