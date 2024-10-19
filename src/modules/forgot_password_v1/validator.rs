use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1GeneratePayload {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1VerifyPayload {
    #[validate(length(min = 6, max = 6))]
    pub code: String,
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1ResetPayload {
    #[validate(length(min = 6, max = 6))]
    pub code: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 4))]
    pub password: String,
    #[validate(length(min = 4))]
    pub confirm_password: String,
}
