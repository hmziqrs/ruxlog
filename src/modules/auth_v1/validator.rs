use serde::{Deserialize, Serialize};
use garde::{self, Validate};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1LoginPayload {
    #[garde(email)]
    email: String,
    #[garde(length(min = 1))]
    password: String,
}

