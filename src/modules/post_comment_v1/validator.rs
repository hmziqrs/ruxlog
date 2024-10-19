use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::models::post_comment::{
    NewPostComment, PostCommentQuery, PostCommentSortBy, UpdatePostComment,
};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreatePostCommentPayload {
    pub post_id: i32,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
}

impl V1CreatePostCommentPayload {
    pub fn into_new_post_comment(self, user_id: i32) -> NewPostComment {
        NewPostComment {
            post_id: self.post_id,
            user_id,
            content: self.content,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdatePostCommentPayload {
    #[validate(length(min = 1, max = 1000))]
    pub content: Option<String>,
}

impl V1UpdatePostCommentPayload {
    pub fn into_update_post_comment(self) -> UpdatePostComment {
        UpdatePostComment {
            content: self.content,
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1PostCommentQueryParams {
    pub page: Option<i64>,
    pub user_id: Option<i32>,
    pub post_id: Option<i32>,
    pub search: Option<String>,
    pub sort_by: Option<PostCommentSortBy>,
    pub sort_order: Option<String>,
}

impl V1PostCommentQueryParams {
    pub fn into_post_comment_query(self) -> PostCommentQuery {
        PostCommentQuery {
            page_no: self.page,
            user_id: self.user_id,
            post_id: self.post_id,
            search: self.search,
            sort_by: self.sort_by,
            sort_order: self.sort_order,
        }
    }
}
