use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use super::PostStatus;

#[derive(Deserialize, Debug)]
pub struct NewPost {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: PostStatus,
    pub user_id: i32,
    pub published_at: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: Option<PostStatus>,
    pub published_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostQuery {
    pub page_no: Option<u64>,
    pub title: Option<String>,
    pub status: Option<PostStatus>,
    pub user_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub published_at: Option<NaiveDateTime>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostWithUser {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: PostStatus,
    pub published_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_id: i32,
    pub user_name: String,
    pub user_avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostWithStats {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: PostStatus,
    pub published_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_id: i32,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub view_count: i64,
    pub comment_count: i64,
}