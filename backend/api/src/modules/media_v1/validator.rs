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

#[cfg(test)]
mod tests {
    use super::*;

    // ── MediaUploadMetadata::apply_field ──────────────────────────────────

    #[test]
    fn apply_field_reference_type_valid() {
        let mut meta = MediaUploadMetadata::default();
        assert!(meta.reference_type.is_none());

        meta.apply_field("reference_type", "category").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::Category));

        meta.apply_field("reference_type", "user").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::User));

        meta.apply_field("reference_type", "post").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::Post));
    }

    #[test]
    fn apply_field_reference_type_empty_clears() {
        let mut meta = MediaUploadMetadata::default();
        meta.reference_type = Some(MediaReference::Category);

        meta.apply_field("reference_type", "  ").unwrap();
        assert!(meta.reference_type.is_none());

        meta.apply_field("reference_type", "").unwrap();
        assert!(meta.reference_type.is_none());
    }

    #[test]
    fn apply_field_reference_type_invalid() {
        let mut meta = MediaUploadMetadata::default();
        let result = meta.apply_field("reference_type", "invalid_type");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid media reference type"));
    }

    #[test]
    fn apply_field_width_valid() {
        let mut meta = MediaUploadMetadata::default();
        assert!(meta.width.is_none());

        meta.apply_field("width", "1920").unwrap();
        assert_eq!(meta.width, Some(1920));

        meta.apply_field("width", "0").unwrap();
        assert_eq!(meta.width, Some(0));

        meta.apply_field("width", "-100").unwrap();
        assert_eq!(meta.width, Some(-100));
    }

    #[test]
    fn apply_field_width_empty_clears() {
        let mut meta = MediaUploadMetadata::default();
        meta.width = Some(800);

        meta.apply_field("width", "").unwrap();
        assert!(meta.width.is_none());
    }

    #[test]
    fn apply_field_width_invalid() {
        let mut meta = MediaUploadMetadata::default();
        let result = meta.apply_field("width", "abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid width"));
    }

    #[test]
    fn apply_field_height_valid() {
        let mut meta = MediaUploadMetadata::default();
        assert!(meta.height.is_none());

        meta.apply_field("height", "1080").unwrap();
        assert_eq!(meta.height, Some(1080));
    }

    #[test]
    fn apply_field_height_empty_clears() {
        let mut meta = MediaUploadMetadata::default();
        meta.height = Some(600);

        meta.apply_field("height", "  ").unwrap();
        assert!(meta.height.is_none());
    }

    #[test]
    fn apply_field_height_invalid() {
        let mut meta = MediaUploadMetadata::default();
        let result = meta.apply_field("height", "not_a_number");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid height"));
    }

    #[test]
    fn apply_field_unknown_field_ignored() {
        let mut meta = MediaUploadMetadata::default();
        // Unknown fields are silently ignored -- no error
        let result = meta.apply_field("unknown_field", "some_value");
        assert!(result.is_ok());
        assert!(meta.reference_type.is_none());
        assert!(meta.width.is_none());
        assert!(meta.height.is_none());
    }

    #[test]
    fn apply_field_whitespace_trimmed() {
        let mut meta = MediaUploadMetadata::default();

        meta.apply_field("width", "  500  ").unwrap();
        assert_eq!(meta.width, Some(500));

        meta.apply_field("reference_type", "  category  ").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::Category));
    }
}
