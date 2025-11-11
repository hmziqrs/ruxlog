use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::post_comment::{CommentQuery, NewComment, UpdateComment};
use crate::utils::SortParam;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreatePostCommentPayload {
    pub post_id: i32,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
}

impl V1CreatePostCommentPayload {
    pub fn into_new_post_comment(self, user_id: i32) -> NewComment {
        NewComment {
            post_id: self.post_id,
            user_id,
            content: self.content,
            likes_count: Some(0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdatePostCommentPayload {
    #[validate(length(min = 1, max = 1000))]
    pub content: Option<String>,
}

impl V1UpdatePostCommentPayload {
    pub fn into_update_post_comment(self) -> UpdateComment {
        UpdateComment {
            content: self.content,
            updated_at: chrono::Utc::now().fixed_offset(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1AdminPostCommentListQuery {
    pub page: Option<u64>,
    pub user_id: Option<i32>,
    pub post_id: Option<i32>,
    pub search: Option<String>,
    pub include_hidden: Option<bool>,
    pub min_flags: Option<i32>,
    pub sorts: Option<Vec<SortParam>>,
    // Date range filters
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

impl V1AdminPostCommentListQuery {
    pub fn into_post_comment_query(self) -> CommentQuery {
        CommentQuery {
            page_no: self.page,
            user_id: self.user_id,
            post_id: self.post_id,
            search_term: self.search,
            include_hidden: self.include_hidden,
            min_flags: self.min_flags,
            sorts: self.sorts,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1AdminModerationPayload {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1FlagCommentPayload {
    #[validate(length(min = 1, max = 500))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1AdminCommentFlagListQuery {
    pub page: Option<u64>,
    pub comment_id: Option<i32>,
    pub user_id: Option<i32>,
    pub search: Option<String>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}
