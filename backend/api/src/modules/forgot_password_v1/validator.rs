use serde::{Deserialize, Serialize};
use validator::Validate;

// The verification-code generators (`forgot_password::Entity::generate_code`
// and `email_verification::Entity::generate_code`) both emit 8-char codes, and
// the password floor elsewhere is 12 (auth_v1 / user_v1). These literals must
// stay in sync with those — `validator`'s `length(min/max)` needs integer
// literals, so a shared const can't be used in the attribute. See Phase 3d/3e.
const CODE_LEN: u64 = 8;
const PASSWORD_MIN: u64 = 12;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1GeneratePayload {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1VerifyPayload {
    #[validate(length(min = CODE_LEN, max = CODE_LEN))]
    pub code: String,
    #[validate(email)]
    pub email: String,
}

/// Response from `verify`. On success the emailed code is **consumed** (the
/// stored row is deleted) and a fresh single-use `reset_token` is issued
/// (audit F#9). The client carries this opaque token into `reset`; the original
/// emailed code is no longer valid for anything, so an interceptor who only had
/// the email loses access once the legitimate user verifies.
#[derive(Debug, Deserialize, Serialize)]
pub struct V1VerifyResponse {
    pub reset_token: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1ResetPayload {
    /// Legacy two-step path: the emailed code. Mutually exclusive with
    /// `reset_token` (the controller picks one). Optional because the
    /// token-based path no longer sends a code. `validator`'s `length` skips
    /// `None`, so this only constrains the code when present.
    #[serde(default)]
    #[validate(length(min = CODE_LEN, max = CODE_LEN))]
    pub code: Option<String>,
    /// Required only for the legacy code path; the reset-token path resolves the
    /// user from the token server-side.
    #[serde(default)]
    #[validate(email)]
    pub email: Option<String>,
    /// Preferred path: the opaque single-use token issued by `verify`.
    #[serde(default)]
    pub reset_token: Option<String>,
    #[validate(length(min = PASSWORD_MIN))]
    pub password: String,
    #[validate(length(min = PASSWORD_MIN))]
    pub confirm_password: String,
}
