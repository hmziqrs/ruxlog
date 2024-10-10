use serde::{Deserialize, Serialize};
use garde::{self, Validate};

use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1LoginPayload {
    #[garde(email)]
    email: String,
    #[garde(length(min = 1))]
    password: String,
}

