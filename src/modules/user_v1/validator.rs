use garde::{self, Validate};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[garde(context(AppState))]
pub struct V1UpdateProfilePayload {
    #[garde(length(min = 1))]
    pub name: Option<String>,
    #[garde(email)]
    pub email: Option<String>,
    #[garde(length(min = 1))]
    pub password: Option<String>,
}
