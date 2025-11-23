use sea_orm::{prelude::DateTimeWithTimeZone, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HiddenFilter {
    All,
    Hidden,
    Visible,
}

impl HiddenFilter {
    /// Resolve an optional filter to a concrete value, defaulting to Visible.
    pub fn resolve(input: Option<Self>) -> Self {
        input.unwrap_or(HiddenFilter::Visible)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentUserMedia {
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
}

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
    pub hidden_filter: Option<HiddenFilter>,
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
            hidden_filter: None,
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
pub struct CommentWithUserJoined {
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
    pub user_avatar_id: Option<i32>,

    // User avatar media fields from join
    pub user_avatar_object_key: Option<String>,
    pub user_avatar_file_url: Option<String>,
    pub user_avatar_mime_type: Option<String>,
    pub user_avatar_width: Option<i32>,
    pub user_avatar_height: Option<i32>,
    pub user_avatar_size: Option<i64>,
}

impl CommentWithUserJoined {
    pub fn into_comment_with_user(self) -> CommentWithUser {
        let avatar = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.user_avatar_id,
            self.user_avatar_object_key,
            self.user_avatar_file_url,
            self.user_avatar_mime_type,
            self.user_avatar_size,
        ) {
            Some(CommentUserMedia {
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

        CommentWithUser {
            id: self.id,
            post_id: self.post_id,
            user_id: self.user_id,
            content: self.content,
            likes_count: self.likes_count,
            hidden: self.hidden,
            flags_count: self.flags_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
            user_name: self.user_name,
            user_avatar: avatar,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_avatar: Option<CommentUserMedia>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentTree {
    pub comment: CommentWithUser,
    pub replies: Vec<CommentWithUser>,
}
