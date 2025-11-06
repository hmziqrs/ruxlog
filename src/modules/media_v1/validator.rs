use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::media::{MediaQuery, MediaReference};
use crate::utils::SortParam;

#[derive(Debug, Default, Deserialize, Serialize, Validate)]
pub struct MediaUploadMetadata {
    pub reference_type: Option<MediaReference>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl MediaUploadMetadata {
    pub fn apply_field(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            "reference_type" => {
                if value.trim().is_empty() {
                    self.reference_type = None;
                } else {
                    self.reference_type = Some(MediaReference::from_str(value.trim())?);
                }
            }
            "width" => {
                if value.trim().is_empty() {
                    self.width = None;
                } else {
                    self.width = Some(
                        value
                            .trim()
                            .parse::<i32>()
                            .map_err(|_| format!("Invalid width: {}", value.trim()))?,
                    );
                }
            }
            "height" => {
                if value.trim().is_empty() {
                    self.height = None;
                } else {
                    self.height = Some(
                        value
                            .trim()
                            .parse::<i32>()
                            .map_err(|_| format!("Invalid height: {}", value.trim()))?,
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1MediaListQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>, // [{ field, order }]
    pub reference_type: Option<MediaReference>,
    pub uploader_id: Option<i32>,
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    // Optional created_at/updated_at range filters (ISO8601)
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

impl V1MediaListQuery {
    pub fn into_query(self) -> MediaQuery {
        MediaQuery {
            page: self.page,
            search: self.search,
            sorts: self.sorts,
            reference_type: self.reference_type,
            uploader_id: self.uploader_id,
            mime_type: self.mime_type,
            extension: self.extension,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1MediaUsageQuery {
    #[validate(length(min = 1, message = "media_ids must contain at least one id"))]
    pub media_ids: Vec<i32>,
}
