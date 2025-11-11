use sea_orm::{prelude::DateTimeWithTimeZone, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FlagUserMedia {
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
}

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

/// Single flag row joined with reporting user fields from query
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult, PartialEq)]
pub struct FlagWithUserJoined {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    // Joined user fields
    pub user_name: String,
    pub user_avatar_id: Option<i32>,
    // User avatar media fields from join
    pub user_avatar_object_key: Option<String>,
    pub user_avatar_file_url: Option<String>,
    pub user_avatar_mime_type: Option<String>,
    pub user_avatar_width: Option<i32>,
    pub user_avatar_height: Option<i32>,
    pub user_avatar_size: Option<i64>,
}

impl FlagWithUserJoined {
    pub fn into_flag_with_user(self) -> FlagWithUser {
        let avatar = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.user_avatar_id,
            self.user_avatar_object_key,
            self.user_avatar_file_url,
            self.user_avatar_mime_type,
            self.user_avatar_size,
        ) {
            Some(FlagUserMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.user_avatar_width,
                height: self.user_avatar_height,
                size,
            })
        } else {
            None
        };

        FlagWithUser {
            id: self.id,
            comment_id: self.comment_id,
            user_id: self.user_id,
            reason: self.reason,
            created_at: self.created_at,
            user_name: self.user_name,
            user_avatar: avatar,
        }
    }
}

/// Single flag row joined with reporting user fields
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FlagWithUser {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    // Joined user fields
    pub user_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_avatar: Option<FlagUserMedia>,
}

/// Aggregate summary for flags on a comment
#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult, PartialEq)]
pub struct FlagsSummary {
    pub comment_id: i32,
    pub flags_count: i64,
}
