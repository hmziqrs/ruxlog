use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewComment {
    pub post_id: i32,
    pub user_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateComment {
    pub content: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CommentQuery {
    pub page_no: Option<u64>,
    pub post_id: i32,
    pub user_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentWithUser {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_name: String,
    pub user_avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentTree {
    pub comment: CommentWithUser,
    pub replies: Vec<CommentWithUser>,
}