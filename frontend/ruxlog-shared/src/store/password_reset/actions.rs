use super::{
    PasswordResetState, RequestResetPayload, ResetPasswordPayload, ResetResult, VerifyResetPayload,
};
use oxcore::http;
use oxstore::state_request_abstraction;

impl PasswordResetState {
    pub async fn request(&self, payload: RequestResetPayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.request,
            Some(meta),
            http::post("/forgot_password/v1/request", &payload).send(),
            "password_reset_request",
            |_resp: &serde_json::Value| (Some(Some(())), None),
        )
        .await;
    }

    pub async fn verify(&self, payload: VerifyResetPayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.verify,
            Some(meta),
            http::post("/forgot_password/v1/verify", &payload).send(),
            "password_reset_verify",
            |resp: &ResetResult| (Some(Some(resp.clone())), None),
        )
        .await;
    }

    pub async fn reset_password(&self, payload: ResetPasswordPayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.reset,
            Some(meta),
            http::post("/forgot_password/v1/reset", &payload).send(),
            "password_reset",
            |resp: &ResetResult| (Some(Some(resp.clone())), None),
        )
        .await;
    }
}
