use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1BlockRoutePayload {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Route pattern must be between 1 and 255 characters"
    ))]
    pub pattern: String,

    #[validate(length(max = 500, message = "Reason must be less than 500 characters"))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateRoutePayload {
    pub is_blocked: bool,

    #[validate(length(max = 500, message = "Reason must be less than 500 characters"))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1RouteStatusQueryParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub is_blocked: Option<bool>,
    pub search: Option<String>,
}

impl Default for V1RouteStatusQueryParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            is_blocked: None,
            search: None,
        }
    }
}
