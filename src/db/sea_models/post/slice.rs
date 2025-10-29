use super::PostStatus;
use chrono::{DateTime, FixedOffset};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

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
    pub sorts: Option<Vec<crate::utils::SortParam>>,
    pub category_id: Option<i32>,
    pub search: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
    // Date range filters
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
    pub published_at_gt: Option<DateTimeWithTimeZone>,
    pub published_at_lt: Option<DateTimeWithTimeZone>,
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
    // Core post data
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
    pub view_count: i32,
    pub likes_count: i32,
    pub category: PostCategory,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub tags: Vec<PostTag>,
    pub author: PostAuthor,

    pub comment_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct PostSitemap {
    pub slug: String,
    pub updated_at: DateTimeWithTimeZone,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTimeWithTimeZone>,
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

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct PostWithJoinedData {
    // Post fields
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: super::PostStatus,
    pub published_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub author_id: i32,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
    pub category_id: i32,

    // Author fields from join
    pub author_name: String,
    pub author_email: String,
    pub author_avatar: Option<String>,

    // Category fields from join
    pub category_name: String,

    pub comment_count: i64,
}

impl PostWithJoinedData {
    pub fn into_relation(&self, tags: Vec<PostTag>) -> PostWithRelations {
        PostWithRelations {
            id: self.id,
            title: self.title.clone(),
            slug: self.slug.clone(),
            content: self.content.clone(),
            excerpt: self.excerpt.clone(),
            featured_image: self.featured_image.clone(),
            status: self.status,
            published_at: self.published_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            view_count: self.view_count,
            likes_count: self.likes_count,
            category: PostCategory {
                id: self.category_id,
                name: self.category_name.clone(),
            },
            tags,
            author: PostAuthor {
                id: self.author_id,
                name: self.author_name.clone(),
                email: self.author_email.clone(),
                avatar: self.author_avatar.clone(),
            },
            comment_count: self.comment_count,
        }
    }
}
