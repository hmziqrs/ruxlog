use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

/// Response for like status check
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LikeStatus {
    pub post_id: i32,
    pub is_liked: bool,
    pub likes_count: i32,
}

/// Response for like/unlike action
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LikeActionResponse {
    pub post_id: i32,
    pub is_liked: bool,
    pub likes_count: i32,
    pub message: String,
}

/// Request to check like status for multiple posts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LikeStatusBatchRequest {
    pub post_ids: Vec<i32>,
}

impl Validate for LikeStatusBatchRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.post_ids.is_empty() {
            errors.add(
                "post_ids",
                ValidationError::new("length")
                    .with_message("post_ids must not be empty".into()),
            );
        }

        if self.post_ids.len() > 100 {
            errors.add(
                "post_ids",
                ValidationError::new("length")
                    .with_message("post_ids must not exceed 100 items".into()),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Response with like status for multiple posts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LikeStatusBatchResponse {
    pub statuses: Vec<LikeStatus>,
}
