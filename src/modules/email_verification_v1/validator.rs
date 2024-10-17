use garde::{self, Validate};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1VerifyPayload {
    #[garde(length(min = 6, max = 6))]
    pub code: String,
}
