use sea_orm::prelude::{DateTimeWithTimeZone, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

fn get_str<'a>(data: &'a Value, field: &str) -> Option<&'a str> {
    data.get(field).and_then(|v| v.as_str())
}

fn get_nested_str<'a>(data: &'a Value, parent: &str, field: &str) -> Option<&'a str> {
    data.get(parent)
        .and_then(|v| v.get(field))
        .and_then(|v| v.as_str())
}

fn non_empty_str(value: Option<&str>) -> bool {
    value.map(|s| !s.trim().is_empty()).unwrap_or(false)
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
                    if !non_empty_str(get_str(&b.data, "text")) {
                        Err(ValidationError::new("paragraph_text_required"))
                    } else {
                        Ok(())
                    }
                }
                "header" => {
                    let text_ok = non_empty_str(get_str(&b.data, "text"));
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
                    let msg_ok = non_empty_str(get_str(&b.data, "message"));
                    let type_ok = get_str(&b.data, "type")
                        .map(|t| matches!(t, "info" | "warning" | "success" | "error"))
                        .unwrap_or(false);
                    if !(msg_ok && type_ok) {
                        Err(ValidationError::new("alert_requires_message_and_valid_type"))
                    } else {
                        Ok(())
                    }
                }
                "quote" => {
                    let text_ok = non_empty_str(get_str(&b.data, "text"));
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
                    let code_ok = get_str(&b.data, "code").map(|s| !s.is_empty()).unwrap_or(false);
                    if !code_ok {
                        Err(ValidationError::new("code_block_code_required"))
                    } else {
                        Ok(())
                    }
                }
                "list" => {
                    let items = b
                        .data
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let has_items = !items.is_empty()
                        && items.iter().all(|it| match it {
                            Value::String(s) => !s.trim().is_empty(),
                            _ => false,
                        });
                    if has_items {
                        Ok(())
                    } else {
                        Err(ValidationError::new("list_items_required"))
                    }
                }
                "delimiter" => Ok(()),
                "image" => {
                    let file_url = get_nested_str(&b.data, "file", "url");
                    let url = get_str(&b.data, "url");
                    let media_id = b
                        .data
                        .get("file")
                        .and_then(|f| f.get("media_id"))
                        .and_then(|v| v.as_i64())
                        .or_else(|| b.data.get("media_id").and_then(|v| v.as_i64()));

                    match (non_empty_str(file_url.or(url)), media_id) {
                        (true, Some(_)) => Ok(()),
                        (false, _) => Err(ValidationError::new("image_url_required")),
                        (_, None) => Err(ValidationError::new("image_media_id_required")),
                    }
                }
                "embed" => {
                    let service_ok = non_empty_str(get_str(&b.data, "service"));
                    let source_ok = non_empty_str(get_str(&b.data, "source"));
                    if service_ok && source_ok {
                        Ok(())
                    } else {
                        Err(ValidationError::new("embed_service_and_source_required"))
                    }
                }
                "linktool" => {
                    if non_empty_str(get_str(&b.data, "link")) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("linktool_link_required"))
                    }
                }
                "attaches" => {
                    if non_empty_str(get_nested_str(&b.data, "file", "url")) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("attaches_url_required"))
                    }
                }
                "raw" => {
                    if non_empty_str(get_str(&b.data, "html")) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("raw_html_required"))
                    }
                }
                "table" => {
                    let content = b
                        .data
                        .get("content")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let has_cells = !content.is_empty()
                        && content.iter().all(|row| {
                            row.as_array()
                                .filter(|cells| !cells.is_empty())
                                .map(|cells| {
                                    cells.iter().all(|cell| match cell {
                                        Value::String(_) => true,
                                        Value::Number(_) => true,
                                        Value::Bool(_) => true,
                                        _ => false,
                                    })
                                })
                                .unwrap_or(false)
                        });
                    if has_cells {
                        Ok(())
                    } else {
                        Err(ValidationError::new("table_content_required"))
                    }
                }
                "warning" => {
                    let title_ok = non_empty_str(get_str(&b.data, "title"));
                    let message_ok = non_empty_str(get_str(&b.data, "message"));
                    if title_ok && message_ok {
                        Ok(())
                    } else {
                        Err(ValidationError::new("warning_title_and_message_required"))
                    }
                }
                "button" => {
                    let text = get_str(&b.data, "text").or_else(|| get_str(&b.data, "buttonText"));
                    let link = get_str(&b.data, "link").or_else(|| get_str(&b.data, "buttonLink"));
                    if non_empty_str(text) && non_empty_str(link) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("button_text_and_link_required"))
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
