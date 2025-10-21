use serde::{Deserialize, Serialize};

use super::MediaReference;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewMedia {
    pub object_key: String,
    pub file_url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size: i64,
    pub extension: Option<String>,
    pub uploader_id: Option<i32>,
    pub reference_type: Option<MediaReference>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MediaDeletion {
    pub id: i32,
}
