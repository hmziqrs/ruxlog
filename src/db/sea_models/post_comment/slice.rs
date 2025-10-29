use sea_orm::{prelude::DateTimeWithTimeZone, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewComment {
    pub post_id: i32,
    pub user_id: i32,
    pub content: String,
    pub likes_count: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateComment {
    pub content: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CommentQuery {
    pub page_no: Option<u64>,
    pub post_id: Option<i32>,
    pub user_id: Option<i32>,
    pub search_term: Option<String>,
    pub include_hidden: Option<bool>,
    pub min_flags: Option<i32>,
    pub sorts: Option<Vec<crate::utils::SortParam>>,
    // Date range filters
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

impl Default for CommentQuery {
    fn default() -> Self {
        Self {
            page_no: None,
            post_id: None,
            user_id: None,
            search_term: None,
            include_hidden: None,
            min_flags: None,
            sorts: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct CommentWithUser {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub content: String,
    pub likes_count: i32,
    pub hidden: bool,
    pub flags_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub user_name: String,
    pub user_avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentTree {
    pub comment: CommentWithUser,
    pub replies: Vec<CommentWithUser>,
}
