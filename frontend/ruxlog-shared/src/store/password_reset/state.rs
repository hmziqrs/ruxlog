use dioxus::prelude::*;
use oxstore::StateFrame;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestResetPayload {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyResetPayload {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResetPasswordPayload {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResetResult {
    pub success: bool,
    pub message: Option<String>,
}

pub struct PasswordResetState {
    pub request: GlobalSignal<StateFrame<Option<()>, RequestResetPayload>>,
    pub verify: GlobalSignal<StateFrame<Option<ResetResult>, VerifyResetPayload>>,
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
