use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewMediaVariant {
    pub media_id: i32,
    pub object_key: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
    pub extension: Option<String>,
    pub quality: Option<i32>,
    pub variant_type: String,
}
