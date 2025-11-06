use super::PostStatus;
use chrono::{DateTime, FixedOffset};
use sea_orm::prelude::{DateTimeWithTimeZone, Json};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorMedia {
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryMedia {
    pub id: i32,
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
}

#[derive(Deserialize, Debug)]
pub struct NewPost {
    pub title: String,
    pub slug: String,
    pub content: Json,
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
    pub content: Option<Json>,
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
    pub slug: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<CategoryMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<CategoryMedia>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostTag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostAuthor {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<AuthorMedia>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostWithRelations {
    // Core post data
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: Json,
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
    pub content: Json,
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
    pub author_avatar_id: Option<i32>,

    // Author avatar media fields from join
    pub author_avatar_object_key: Option<String>,
    pub author_avatar_file_url: Option<String>,
    pub author_avatar_mime_type: Option<String>,
    pub author_avatar_width: Option<i32>,
    pub author_avatar_height: Option<i32>,
    pub author_avatar_size: Option<i64>,

    // Category fields from join
    pub category_name: String,
    pub category_slug: String,
    pub category_color: String,
    pub category_cover_id: Option<i32>,
    pub category_logo_id: Option<i32>,

    // Category cover media fields
    pub category_cover_object_key: Option<String>,
    pub category_cover_file_url: Option<String>,
    pub category_cover_mime_type: Option<String>,
    pub category_cover_width: Option<i32>,
    pub category_cover_height: Option<i32>,
    pub category_cover_size: Option<i64>,

    // Category logo media fields
    pub category_logo_object_key: Option<String>,
    pub category_logo_file_url: Option<String>,
    pub category_logo_mime_type: Option<String>,
    pub category_logo_width: Option<i32>,
    pub category_logo_height: Option<i32>,
    pub category_logo_size: Option<i64>,

    pub comment_count: i64,
}

impl PostWithJoinedData {
    pub fn into_relation(&self, tags: Vec<PostTag>) -> PostWithRelations {
        let avatar = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.author_avatar_id,
            self.author_avatar_object_key.clone(),
            self.author_avatar_file_url.clone(),
            self.author_avatar_mime_type.clone(),
            self.author_avatar_size,
        ) {
            Some(AuthorMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.author_avatar_width,
                height: self.author_avatar_height,
                size,
            })
        } else {
            None
        };

        let category_cover = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.category_cover_id,
            self.category_cover_object_key.clone(),
            self.category_cover_file_url.clone(),
            self.category_cover_mime_type.clone(),
            self.category_cover_size,
        ) {
            Some(CategoryMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.category_cover_width,
                height: self.category_cover_height,
                size,
            })
        } else {
            None
        };

        let category_logo = if let (Some(id), Some(key), Some(url), Some(mime), Some(size)) = (
            self.category_logo_id,
            self.category_logo_object_key.clone(),
            self.category_logo_file_url.clone(),
            self.category_logo_mime_type.clone(),
            self.category_logo_size,
        ) {
            Some(CategoryMedia {
                id,
                object_key: key,
                file_url: url,
                mime_type: mime,
                width: self.category_logo_width,
                height: self.category_logo_height,
                size,
            })
        } else {
            None
        };

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
                slug: self.category_slug.clone(),
                color: self.category_color.clone(),
                cover: category_cover,
                logo: category_logo,
            },
            tags,
            author: PostAuthor {
                id: self.author_id,
                name: self.author_name.clone(),
                email: self.author_email.clone(),
                avatar,
            },
            comment_count: self.comment_count,
        }
    }
}
