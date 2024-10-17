use garde::{self, Validate};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1GeneratePayload {
    #[garde(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1VerifyPayload {
    #[garde(length(min = 6, max = 6))]
    pub code: String,
    #[garde(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1ResetPayload {
    #[garde(length(min = 6, max = 6))]
    pub code: String,
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 4))]
    pub password: String,
    #[garde(length(min = 4))]
    pub confirm_password: String,
}
