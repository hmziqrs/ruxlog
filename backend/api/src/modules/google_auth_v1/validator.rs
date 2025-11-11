use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct GoogleCallbackQuery {
    #[validate(length(min = 1))]
    pub code: String,
    #[validate(length(min = 1))]
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct GoogleExchangeRequest {
    #[validate(length(min = 1))]
    pub code: String,
    #[validate(length(min = 1))]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub verified_email: bool,
}
