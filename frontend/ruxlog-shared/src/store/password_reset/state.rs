use dioxus::prelude::*;
use oxstore::StateFrame;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestResetPayload {
    pub email: String,
}

/// Request body for the verify step. Matches the backend `V1VerifyPayload`.
/// NOTE: the backend reads `code` (not `token`) — the legacy payload here sent
/// `token` and never matched the backend, so the field names are corrected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyResetPayload {
    pub email: String,
    pub code: String,
}

/// Response from the verify step (backend `V1VerifyResponse`). On success the
/// emailed code is **consumed** (single-use) and this opaque token is the only
/// credential that can subsequently reset the password (audit F#9). The UI must
/// carry `reset_token` into [`ResetPasswordPayload::reset_token`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyResetResponse {
    pub reset_token: String,
}

/// Request body for the reset step. Preferred path: `reset_token` (issued by
/// verify). The backend also accepts the legacy `{ email, code }` pair, but the
/// two-step UI always goes through verify first, so it carries the token rather
/// than re-sending the now-dead code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResetPasswordPayload {
    pub reset_token: String,
    pub password: String,
    pub confirm_password: String,
}

/// Outcome of the reset step. `success` defaults so the backend's
/// `{ "message": "..." }` response deserializes cleanly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResetResult {
    #[serde(default)]
    pub success: bool,
    pub message: Option<String>,
}

pub struct PasswordResetState {
    pub request: GlobalSignal<StateFrame<Option<()>, RequestResetPayload>>,
    pub verify: GlobalSignal<StateFrame<Option<VerifyResetResponse>, VerifyResetPayload>>,
    pub reset: GlobalSignal<StateFrame<Option<ResetResult>, ResetPasswordPayload>>,
}

impl PasswordResetState {
    pub fn new() -> Self {
        Self {
            request: GlobalSignal::new(|| StateFrame::new()),
            verify: GlobalSignal::new(|| StateFrame::new()),
            reset: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub fn reset(&self) {
        *self.request.write() = StateFrame::new();
        *self.verify.write() = StateFrame::new();
        *self.reset.write() = StateFrame::new();
    }
}

static PASSWORD_RESET_STATE: OnceLock<PasswordResetState> = OnceLock::new();

pub fn use_password_reset() -> &'static PasswordResetState {
    PASSWORD_RESET_STATE.get_or_init(PasswordResetState::new)
}
