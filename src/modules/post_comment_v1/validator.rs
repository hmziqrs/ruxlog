use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::post_comment::{CommentQuery, NewComment, UpdateComment};

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

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1PostCommentQueryParams {
    pub page: Option<u64>,
    pub user_id: Option<i32>,
    pub post_id: Option<i32>,
    pub search: Option<String>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

impl V1PostCommentQueryParams {
    pub fn into_post_comment_query(self) -> CommentQuery {
        CommentQuery {
            page_no: self.page,
            user_id: self.user_id,
            post_id: self.post_id,
            search_term: self.search,
            sort_by: self.sort_by,
            sort_order: self.sort_order,
        }
    }
}
