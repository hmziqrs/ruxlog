// use bool;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::models::post::{NewPost, PostSortBy, UpdatePost};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreatePostPayload {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
    pub published_at: Option<NaiveDateTime>,
    #[serde(default = "bool::default")]
    pub is_published: bool,
    #[validate(length(min = 1, max = 255))]
    pub slug: String,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub category_id: Option<i32>,
}

impl V1CreatePostPayload {
    pub fn into_new_post(self, author_id: i32) -> NewPost {
        NewPost {
            title: self.title,
            content: self.content,
            author_id,
            published_at: self.published_at,
            is_published: self.is_published,
            slug: self.slug,
            excerpt: self.excerpt,
            featured_image_url: self.featured_image_url,
            category_id: self.category_id,
            view_count: 0,
            likes_count: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdatePostPayload {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1))]
    pub content: Option<String>,
    pub published_at: Option<Option<NaiveDateTime>>,
    pub is_published: Option<bool>,
    #[validate(length(min = 1, max = 255))]
    pub slug: Option<String>,
    #[validate(length(max = 500))]
    pub excerpt: Option<Option<String>>,
    pub featured_image_url: Option<Option<String>>,
    pub category_id: Option<Option<i32>>,
}

impl V1UpdatePostPayload {
    pub fn into_update_post(self, author_id: i32) -> UpdatePost {
        UpdatePost {
            title: self.title,
            content: self.content,
            author_id: Some(author_id),
            published_at: self.published_at,
            updated_at: chrono::Utc::now().naive_utc(),
            is_published: self.is_published,
            slug: self.slug,
            excerpt: self.excerpt,
            featured_image_url: self.featured_image_url,
            category_id: self.category_id,
            view_count: None,
            likes_count: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1PostQueryParams {
    pub page: Option<i64>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<PostSortBy>,
    pub sort_order: Option<String>,
}
