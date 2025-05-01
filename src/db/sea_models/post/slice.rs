use sea_orm::prelude::DateTimeWithTimeZone;
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
    pub author_id: i32,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub category_id: i32,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: Option<PostStatus>,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub updated_at: DateTimeWithTimeZone,
    pub category_id: Option<i32>,
    pub view_count: Option<i32>,
    pub likes_count: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostQuery {
    pub page_no: Option<u64>,
    pub title: Option<String>,
    pub status: Option<PostStatus>,
    pub author_id: Option<i32>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
    pub category_id: Option<i32>,
    pub search: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
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
    pub published_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub author_id: i32,
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
    pub published_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub author_id: i32,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub view_count: i64,
    pub comment_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostTag {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostAuthor {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostWithRelations {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: PostStatus,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub author_id: i32,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
    pub category: Option<PostCategory>,
    pub tags: Vec<PostTag>,
    pub author: PostAuthor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostSitemap {
    pub slug: String,
    pub updated_at: DateTimeWithTimeZone,
    pub published_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PostSortBy {
    Title,
    UpdatedAt,
    PublishedAt,
    ViewCount,
    LikesCount,
}