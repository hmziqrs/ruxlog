use super::{
    AuthState, AuthUser, LoginPayload, LoginTotpPayload, RegisterPayload, TwoFactorSetup,
    TwoFactorVerifyPayload, UserRole, UserSession,
};
use crate::store::{
    use_categories, use_comments, use_email_verification, use_media, use_password_reset, use_post,
    use_tag,
};

#[cfg(feature = "analytics-store")]
use crate::store::use_analytics;

#[cfg(feature = "admin-routes-store")]
use crate::store::use_admin_routes;

#[cfg(feature = "newsletter-store")]
use crate::store::use_newsletter;

#[cfg(feature = "users-store")]
use crate::store::use_user;

#[cfg(feature = "image-editor")]
use crate::store::use_image_editor;
use dioxus::{logger::tracing, prelude::*};
use oxcore::http;
use oxstore::{state_request_abstraction, StateFrame};

/// Body of the `totp_required` login response (F#4/F#7/F#16 two-step login).
/// A 200 with this shape means a correct password was accepted but the user
/// has 2FA enrolled, so NO session was issued — the caller must POST
/// `totp_token` + a TOTP code to `/auth/v1/login/totp`.
#[derive(Debug, Clone, serde::Deserialize)]
struct TotpRequiredResponse {
    status: String,
    totp_token: String,
}

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
            register_status: GlobalSignal::new(|| StateFrame::new()),
            init_status: GlobalSignal::new(|| StateFrame::new()),
            two_factor: GlobalSignal::new(|| StateFrame::new()),
            sessions: GlobalSignal::new(|| StateFrame::new()),
            login_totp: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub async fn logout(&self) {
        self.logout_status.write().set_loading();
        let empty_body = {};
        let result = http::post("/auth/v1/log_out", &empty_body).send().await;
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    self.logout_status.write().set_success(None);
                    *self.user.write() = None;
                    self.reset_all_stores();
                    // The CSRF token is HMAC-bound to the session id of the
                    // session we just destroyed, so it is now invalid — the
                    // per-session guard would reject any mutating request that
                    // still carried it (audit F#17). Drop it, then fetch a
                    // fresh token bound to the new anonymous session that
                    // `/csrf/v1/generate` materializes. This mirrors the login
                    // flow and keeps the client's token always matched to the
                    // active session across the full login ⇄ logout cycle.
                    http::set_csrf_token("");
                    let _ = http::refresh_csrf_token().await;
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
        // Always available stores
        use_categories().reset();
        use_tag().reset();
        use_media().reset();
        use_post().reset();
        use_comments().reset();
        use_email_verification().reset();
        use_password_reset().reset();

        // Feature-gated stores
        #[cfg(feature = "image-editor")]
        use_image_editor().reset();

        #[cfg(feature = "analytics-store")]
        use_analytics().reset();

        #[cfg(feature = "admin-routes-store")]
        use_admin_routes().reset();

        #[cfg(feature = "newsletter-store")]
        use_newsletter().reset();

        #[cfg(feature = "users-store")]
        use_user().reset();
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
        // Clear any stale pending-TOTP step from a previous 2FA login attempt.
        *self.login_totp.write() = StateFrame::new();
        let payload = LoginPayload { email, password };
        let result = http::post("/auth/v1/log_in", &payload).send().await;
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let raw = response.body_text();

                    // F#4/F#7/F#16 (two-step 2FA-at-login): a 200 whose body
                    // is `{ status: "totp_required", totp_token }` means the
                    // user has 2FA enrolled and a correct password is NOT
                    // enough — no session was issued. Park the opaque pending
                    // token in `login_totp` (the login screen then shows a code
                    // input) and surface a non-failed status so the user is
                    // prompted rather than shown an error. The token
                    // authenticates ONLY the TOTP step.
                    if let Ok(totp_req) = serde_json::from_str::<TotpRequiredResponse>(&raw) {
                        if totp_req.status == "totp_required" {
                            self.login_totp
                                .write()
                                .set_success(Some(totp_req.totp_token));
                            self.login_status.write().set_success(None);
                            return;
                        }
                    }

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
                            // Login may rotate the server-side session; re-fetch
                            // the per-session CSRF token so subsequent mutating
                            // requests carry one bound to the active session.
                            let _ = http::refresh_csrf_token().await;
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

    pub async fn register(&self, name: String, email: String, password: String) {
        self.register_status.write().set_loading();
        let payload = RegisterPayload {
            name,
            email: email.clone(),
            password: password.clone(),
        };
        let result = http::post("/auth/v1/register", &payload).send().await;
        match result {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    self.register_status.write().set_success(None);
                    // Auto-login after successful registration
                    self.login(email, password).await;
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    self.register_status.write().set_api_error(status, body);
                }
            }
            Err(e) => {
                let (kind, msg) = oxstore::error::classify_transport_error(&e);
                self.register_status
                    .write()
                    .set_transport_error(kind, Some(msg));
            }
        }
    }

    pub fn reset(&self) {
        *self.user.write() = None;
        *self.login_status.write() = StateFrame::new();
        *self.logout_status.write() = StateFrame::new();
        *self.register_status.write() = StateFrame::new();
        *self.init_status.write() = StateFrame::new();
        *self.two_factor.write() = StateFrame::new();
        *self.sessions.write() = StateFrame::new();
        *self.login_totp.write() = StateFrame::new();
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

    /// Second step of the two-step 2FA-at-login flow (F#4/F#7/F#16). Submits
    /// the pending `totp_token` (from `login()`'s `totp_required` response)
    /// plus the user's 6-digit code to `/auth/v1/login/totp`. On success the
    /// server issues the full session and returns the user; this clears the
    /// pending-TOTP step and records the logged-in user exactly as `login()`
    /// does on a non-2FA success. On failure the pending step is left in place
    /// so the user can retry (the server's `totp:{user_id}` abuse-limiter
    /// bounds the attempts).
    pub async fn verify_login_totp(&self, totp_token: String, code: String) {
        self.login_status.write().set_loading();
        let payload = LoginTotpPayload { totp_token, code };
        let result = http::post("/auth/v1/login/totp", &payload).send().await;
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
                            // The pending token was single-use and is now
                            // consumed server-side; clear it client-side.
                            *self.login_totp.write() = StateFrame::new();
                            self.login_status.write().set_success(None);
                            let _ = http::refresh_csrf_token().await;
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
            |sessions: &Vec<UserSession>| (Some(sessions.clone()), None),
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
