use sea_orm::{prelude::DateTimeWithTimeZone, FromQueryResult};
use serde::{Deserialize, Serialize};

/// New flag to be created for a comment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewCommentFlag {
    pub comment_id: i32,
    pub user_id: i32,
    pub reason: Option<String>,
}

/// Query parameters for listing comment flags
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CommentFlagQuery {
    pub page_no: Option<u64>,
    pub comment_id: Option<i32>,
    pub user_id: Option<i32>,
    /// Optional search term against reason
    pub search_term: Option<String>,
    /// e.g. ["created_at"]
    pub sort_by: Option<Vec<String>>,
    /// "asc" or "desc"
    pub sort_order: Option<String>,
}

impl Default for CommentFlagQuery {
    fn default() -> Self {
        Self {
            page_no: None,
            comment_id: None,
            user_id: None,
            search_term: None,
            sort_by: None,
            sort_order: None,
        }
    }
}

/// Single flag row joined with reporting user fields
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult, PartialEq)]
pub struct FlagWithUser {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    // Joined user fields
    pub user_name: String,
    pub user_avatar: Option<String>,
}

/// Aggregate summary for flags on a comment
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult, PartialEq)]
pub struct FlagsSummary {
    pub comment_id: i32,
    pub flags_count: i64,
}
