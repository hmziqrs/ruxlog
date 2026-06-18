use serde::{Deserialize, Serialize};
use validator::Validate;

// Must match `email_verification::Entity::generate_code`, which emits 8 chars
// (~47 bits). Keep in sync — see Phase 3d.
const CODE_LEN: u64 = 8;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1VerifyPayload {
    #[validate(length(min = CODE_LEN, max = CODE_LEN))]
    pub code: String,
}
