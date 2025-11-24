use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpsertConstantRequest {
    #[validate(length(min = 1, max = 191))]
    pub key: String,
    #[validate(length(min = 0))]
    pub value: String,
    #[validate(length(max = 50))]
    pub value_type: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub is_sensitive: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ConstantsListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub is_sensitive: Option<bool>,
    pub value_type: Option<String>,
}
