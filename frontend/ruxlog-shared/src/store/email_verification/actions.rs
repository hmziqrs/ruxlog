use super::{
    EmailVerificationState, ResendVerificationPayload, VerificationResult, VerifyEmailPayload,
};
use oxcore::http;
use oxstore::state_request_abstraction;

impl EmailVerificationState {
    pub async fn verify(&self, payload: VerifyEmailPayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.verify,
            Some(meta),
            http::post("/email_verification/v1/verify", &payload).send(),
            "email_verification",
            |resp: &VerificationResult| (Some(Some(resp.clone())), None),
        )
        .await;
    }

    pub async fn resend(&self, payload: ResendVerificationPayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.resend,
            Some(meta),
            http::post("/email_verification/v1/resend", &payload).send(),
            "email_verification_resend",
            |_resp: &serde_json::Value| (Some(Some(())), None),
        )
        .await;
    }
}
