use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::media::MediaReference;

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
