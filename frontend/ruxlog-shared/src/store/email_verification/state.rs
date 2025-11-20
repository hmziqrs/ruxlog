use dioxus::prelude::*;
use oxstore::StateFrame;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyEmailPayload {
    pub code: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResendVerificationPayload {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VerificationResult {
    pub success: bool,
    pub message: Option<String>,
}

pub struct EmailVerificationState {
    pub verify: GlobalSignal<StateFrame<Option<VerificationResult>, VerifyEmailPayload>>,
    pub resend: GlobalSignal<StateFrame<Option<()>, ResendVerificationPayload>>,
}

impl EmailVerificationState {
    pub fn new() -> Self {
        Self {
            verify: GlobalSignal::new(|| StateFrame::new()),
            resend: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub fn reset(&self) {
        *self.verify.write() = StateFrame::new();
        *self.resend.write() = StateFrame::new();
    }
}

static EMAIL_VERIFICATION_STATE: OnceLock<EmailVerificationState> = OnceLock::new();

pub fn use_email_verification() -> &'static EmailVerificationState {
    EMAIL_VERIFICATION_STATE.get_or_init(EmailVerificationState::new)
}
