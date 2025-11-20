use super::{
    AuthState, AuthUser, LoginPayload, TwoFactorSetup, TwoFactorVerifyPayload, UserRole,
    UserSession,
};
use crate::store::{
    /* use_admin_routes, */ use_analytics, use_categories, use_comments, use_email_verification,
    use_image_editor, use_media, use_newsletter, use_password_reset, use_post, use_tag, use_user,
};
use dioxus::{logger::tracing, prelude::*};
use oxcore::http;
use oxstore::{state_request_abstraction, StateFrame};

impl AuthUser {
    pub fn new(id: i32, name: String, email: String, role: UserRole, is_verified: bool) -> Self {
        AuthUser {
            id,
            name,
            email,
            avatar: None,
            role,
            is_verified,
        }
    }

    pub fn dev() -> Self {
        AuthUser::new(
            1,
            "Dev User".to_string(),
            "dev@example.com".to_string(),
            UserRole::Admin,
            true,
        )
    }
}

impl AuthState {
    pub fn new() -> Self {
        AuthState {
            user: GlobalSignal::new(|| None),
            login_status: GlobalSignal::new(|| StateFrame::new()),
            logout_status: GlobalSignal::new(|| StateFrame::new()),
            init_status: GlobalSignal::new(|| StateFrame::new()),
            two_factor: GlobalSignal::new(|| StateFrame::new()),
            sessions: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub async fn logout(&self) {
        self.logout_status.write().set_loading();
        let empty_body = {};
        let result = http::post("/auth/v1/log_out", &empty_body)
            .send()
            .await;
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    self.logout_status.write().set_success(None);
                    *self.user.write() = None;
                    self.reset_all_stores();
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    self.logout_status.write().set_api_error(status, body);
                    *self.user.write() = None;
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                self.logout_status
                    .write()
                    .set_transport_error(kind, Some(msg));
                *self.user.write() = None;
            }
        }
    }

    fn reset_all_stores(&self) {
        use_categories().reset();
        use_tag().reset();
        use_user().reset();
        use_media().reset();
        use_post().reset();
        use_analytics().reset();
        use_image_editor().reset();
        use_comments().reset();
        use_newsletter().reset();
        use_email_verification().reset();
        use_password_reset().reset();
        // use_admin_routes().reset(); // TODO: Fix admin_routes compilation errors
    }

    pub async fn init(&self) {
        // self.init_status.write().set_success(None, None);
        // *self.user.write() = Some(User::dev());
        self.init_status.write().set_loading();
        let result = http::get("/user/v1/get").send().await;
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let raw = response.body_text();
                    match serde_json::from_str::<AuthUser>(&raw) {
                        Ok(user) => {
                            if !user.is_verified || !user.is_admin() {
                                self.init_status.write().set_failed(
                                    "User not allowed to access this page.".to_string(),
                                );
                                return;
                            }
                            *self.user.write() = Some(user);
                            self.init_status.write().set_success(None);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse user data: {}\nResponse: {}", e, raw);
                            self.init_status.write().set_decode_error(
                                "user",
                                format!("{}", e),
                                Some(raw),
                            );
                        }
                    }
                } else if response.status() == 401 {
                    // Unauthorized, no user logged in
                    self.init_status.write().set_success(None);
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    self.init_status.write().set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                self.init_status
                    .write()
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    pub async fn login(&self, email: String, password: String) {
        self.login_status.write().set_loading();
        let payload = LoginPayload { email, password };
        let result = http::post("/auth/v1/log_in", &payload).send().await;
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let raw = response.body_text();
                    match serde_json::from_str::<AuthUser>(&raw) {
                        Ok(user) => {
                            if !user.is_verified || !user.is_admin() {
                                self.login_status.write().set_failed(
                                    "User not allowed to access this page.".to_string(),
                                );
                                return;
                            }
                            *self.user.write() = Some(user);
                            self.login_status.write().set_success(None);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse user data: {}\nResponse: {}", e, raw);
                            self.login_status.write().set_decode_error(
                                "user",
                                format!("{}", e),
                                Some(raw),
                            );
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    self.login_status.write().set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                self.login_status
                    .write()
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    pub fn reset(&self) {
        *self.user.write() = None;
        *self.login_status.write() = StateFrame::new();
        *self.logout_status.write() = StateFrame::new();
        *self.init_status.write() = StateFrame::new();
        *self.two_factor.write() = StateFrame::new();
        *self.sessions.write() = StateFrame::new();
    }
}

// =============================================================================
// Two-Factor Authentication
// =============================================================================

impl AuthState {
    pub async fn setup_2fa(&self) {
        let _ = state_request_abstraction(
            &self.two_factor,
            None::<()>,
            http::post("/auth/v1/2fa/setup", &serde_json::json!({})).send(),
            "two_factor_setup",
            |payload: &TwoFactorSetup| (Some(Some(payload.clone())), None),
        )
        .await;
    }

    pub async fn verify_2fa(&self, payload: TwoFactorVerifyPayload) {
        let _ = state_request_abstraction(
            &self.two_factor,
            None::<()>,
            http::post("/auth/v1/2fa/verify", &payload).send(),
            "two_factor_verify",
            |_resp: &serde_json::Value| (Some(None), None),
        )
        .await;
    }

    pub async fn disable_2fa(&self, payload: TwoFactorVerifyPayload) {
        let _ = state_request_abstraction(
            &self.two_factor,
            None::<()>,
            http::post("/auth/v1/2fa/disable", &payload).send(),
            "two_factor_disable",
            |_resp: &serde_json::Value| (Some(None), None),
        )
        .await;
    }
}

// =============================================================================
// Session Management
// =============================================================================

impl AuthState {
    pub async fn list_sessions(&self) {
        let _ = state_request_abstraction(
            &self.sessions,
            None::<()>,
            http::post("/auth/v1/sessions/list", &serde_json::json!({})).send(),
            "user_sessions",
            |sessions: &Vec<UserSession>| (Some(Some(sessions.clone())), None),
        )
        .await;
    }

    pub async fn terminate_session(&self, session_id: String) {
        let _ = state_request_abstraction(
            &self.sessions,
            None::<()>,
            http::post(
                &format!("/auth/v1/sessions/terminate/{}", session_id),
                &serde_json::json!({}),
            )
            .send(),
            "terminate_session",
            |_resp: &serde_json::Value| (None, None),
        )
        .await;

        self.list_sessions().await;
    }
}
