use sea_orm::prelude::{DateTimeWithTimeZone, Json};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::db::sea_models::post::{NewPost, PostQuery, PostStatus, UpdatePost};
use crate::utils::SortParam;

// Validated Editor.js document types
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditorJsDocument {
    pub time: Option<i64>,
    pub blocks: Vec<EditorJsBlock>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditorJsBlock {
    #[serde(rename = "type")]
    pub kind: String,
    pub data: serde_json::Value,
}

impl Validate for EditorJsDocument {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.blocks.is_empty() {
            errors.add("blocks", ValidationError::new("blocks_empty"));
            return Err(errors);
        }

        for b in self.blocks.iter() {
            let res: Result<(), ValidationError> = match b.kind.as_str() {
                "paragraph" => {
                    let text = b.data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    if text.trim().is_empty() {
                        Err(ValidationError::new("paragraph_text_required"))
                    } else {
                        Ok(())
                    }
                }
                "header" => {
                    let text_ok = b
                        .data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false);
                    let level_ok = b
                        .data
                        .get("level")
                        .and_then(|v| v.as_i64())
                        .map(|l| (1..=6).contains(&l))
                        .unwrap_or(false);
                    if !(text_ok && level_ok) {
                        Err(ValidationError::new("header_requires_text_and_level_1_6"))
                    } else {
                        Ok(())
                    }
                }
                "alert" => {
                    let msg_ok = b
                        .data
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false);
                    let type_ok = b
                        .data
                        .get("type")
                        .and_then(|v| v.as_str())
                        .map(|t| matches!(t, "info" | "warning" | "success" | "error"))
                        .unwrap_or(false);
                    if !(msg_ok && type_ok) {
                        Err(ValidationError::new("alert_requires_message_and_valid_type"))
                    } else {
                        Ok(())
                    }
                }
                "quote" => {
                    let text_ok = b
                        .data
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false);
                    if !text_ok {
                        Err(ValidationError::new("quote_text_required"))
                    } else {
                        Ok(())
                    }
                }
                "checklist" => {
                    let items = b
                        .data
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    if items.is_empty() {
                        Err(ValidationError::new("checklist_items_required"))
                    } else {
                        let mut bad = None;
                        for it in items.iter() {
                            let text_ok = it
                                .get("text")
                                .and_then(|v| v.as_str())
                                .map(|s| !s.trim().is_empty())
                                .unwrap_or(false);
                            if !text_ok {
                                bad = Some("checklist_item_text_required");
                                break;
                            }
                        }
                        if let Some(kind) = bad {
                            Err(ValidationError::new(kind))
                        } else {
                            Ok(())
                        }
                    }
                }
                "code" => {
                    let code_ok = b
                        .data
                        .get("code")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                    if !code_ok {
                        Err(ValidationError::new("code_block_code_required"))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(ValidationError::new("unsupported_block_type")),
            };

            if let Err(e) = res {
                errors.add("blocks", e);
                return Err(errors);
            }
        }
        Ok(())
    }
}

impl EditorJsDocument {
    pub fn into_json(self) -> Json {
        serde_json::to_value(self).unwrap_or(serde_json::json!({
            "time": 0,
            "blocks": [],
            "version": "2.30.7"
        }))
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1CreatePostPayload {
    #[validate(length(min = 3, max = 255))]
    pub title: String,
    #[validate(nested)]
    pub content: EditorJsDocument,
    pub published_at: Option<DateTimeWithTimeZone>,
    #[serde(default)]
    pub is_published: bool,
    #[validate(length(min = 3, max = 255))]
    pub slug: String,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub category_id: i32,
    #[serde(default = "Vec::new")]
    pub tag_ids: Vec<i32>,
}

impl V1CreatePostPayload {
    pub fn into_new_post(self, author_id: i32) -> NewPost {
        NewPost {
            title: self.title,
            content: self.content.into_json(),
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
    #[validate(length(min = 3, max = 255))]
    pub title: Option<String>,
    #[validate(nested)]
    pub content: Option<EditorJsDocument>,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub status: Option<PostStatus>,
    #[validate(length(min = 3, max = 255))]
    pub slug: Option<String>,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub category_id: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
}

impl V1UpdatePostPayload {
    pub fn into_update_post(self) -> UpdatePost {
        UpdatePost {
            title: self.title,
            content: self.content.map(|d| d.into_json()),
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
    pub sorts: Option<Vec<SortParam>>,
    pub tag_ids: Option<Vec<i32>>,
    pub title: Option<String>,
    // Date range filters
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
    pub published_at_gt: Option<DateTimeWithTimeZone>,
    pub published_at_lt: Option<DateTimeWithTimeZone>,
}

impl V1PostQueryParams {
    pub fn into_post_query(self) -> PostQuery {
        PostQuery {
            page_no: self.page,
            author_id: self.author_id,
            category_id: self.category_id,
            status: self.status,
            search: self.search,
            sorts: self.sorts,
            tag_ids: self.tag_ids,
            title: self.title,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
            published_at_gt: self.published_at_gt,
            published_at_lt: self.published_at_lt,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1AutosavePayload {
    pub post_id: i32,
    #[validate(nested)]
    pub content: EditorJsDocument,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1SchedulePayload {
    pub post_id: i32,
    pub publish_at: DateTimeWithTimeZone,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1SeriesCreatePayload {
    #[validate(length(min = 3, max = 255))]
    pub name: String,
    #[validate(length(min = 3, max = 255))]
    pub slug: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1SeriesUpdatePayload {
    #[validate(length(min = 3, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 3, max = 255))]
    pub slug: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1SeriesListQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
}
