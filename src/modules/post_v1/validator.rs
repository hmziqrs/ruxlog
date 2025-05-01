use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::post::{NewPost, PostQuery, PostStatus, UpdatePost};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreatePostPayload {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
    pub published_at: Option<DateTimeWithTimeZone>,
    #[serde(default)]
    pub is_published: bool,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub category_id: Option<i32>,
    #[serde(default = "Vec::new")]
    pub tag_ids: Vec<i32>,
}

impl V1CreatePostPayload {
    pub fn into_new_post(self, author_id: i32) -> NewPost {
        NewPost {
            title: self.title,
            content: self.content,
            author_id: author_id,
            published_at: self.published_at,
            status: if self.is_published {
                PostStatus::Published
            } else {
                PostStatus::Draft
            },
            slug: self.slug,
            excerpt: self.excerpt,
            featured_image: self.featured_image,
            category_id: self.category_id,
            view_count: 0,
            likes_count: 0,
            tag_ids: self.tag_ids,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdatePostPayload {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1))]
    pub content: Option<String>,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub status: Option<PostStatus>,
    #[validate(length(min = 1, max = 255))]
    pub slug: Option<String>,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub category_id: Option<Option<i32>>,
    pub tag_ids: Option<Vec<i32>>,
}

impl V1UpdatePostPayload {
    pub fn into_update_post(self) -> UpdatePost {
        UpdatePost {
            title: self.title,
            content: self.content,
            // author_id: Some(author_id),
            published_at: self.published_at,
            updated_at: chrono::Utc::now().fixed_offset(),
            status: self.status,
            slug: self.slug,
            excerpt: self.excerpt,
            featured_image: self.featured_image,
            category_id: self.category_id,
            view_count: None,
            likes_count: None,
            tag_ids: self.tag_ids,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1PostQueryParams {
    pub page: Option<u64>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub status: Option<PostStatus>,
    pub search: Option<String>,
    pub sort_by: Option<Vec<String>>,
    pub sort_order: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
    pub title: Option<String>,
}

impl V1PostQueryParams {
    pub fn into_post_query(self) -> PostQuery {
        PostQuery {
            page_no: self.page,
            author_id: self.author_id,
            category_id: self.category_id,
            status: self.status,
            search: self.search,
            sort_by: self.sort_by,
            sort_order: self.sort_order,
            tag_ids: self.tag_ids,
            title: self.title,
            created_at: None,
            updated_at: None,
            published_at: None,
        }
    }
}
