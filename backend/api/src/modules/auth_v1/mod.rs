pub mod controller;
pub mod validator;

use axum::{middleware, routing::post, Router};

use crate::{middlewares::auth_guard, AppState};

pub fn routes() -> Router<AppState> {
    let mut public = Router::<AppState>::new()
        .route("/log_in", post(controller::log_in))
        // Second step of the two-step 2FA-at-login flow (F#4/F#7/F#16): a
        // correct password no longer yields a full session for 2FA-enrolled
        // users — they get a short-lived `totp_token` and must POST it here
        // with a valid TOTP code. Public (the token IS the auth for this step);
        // brute-force on the 6-digit code is bounded by the `totp:{user_id}`
        // abuse-limiter bucket invoked inside the handler.
        .route("/login/totp", post(controller::login_totp));

    #[cfg(feature = "user-management")]
    {
        public = public.route("/register", post(controller::register));
    }

    let public = public.route_layer(middleware::from_fn(auth_guard::unauthenticated));

    let mut authenticated = Router::<AppState>::new().route("/log_out", post(controller::log_out));

    #[cfg(feature = "auth-2fa")]
    {
        authenticated = authenticated
            .route("/2fa/setup", post(controller::twofa_setup))
            .route("/2fa/verify", post(controller::twofa_verify))
            .route("/2fa/disable", post(controller::twofa_disable));
    }

    let authenticated = authenticated
        .route("/sessions/list", post(controller::sessions_list))
        .route(
            "/sessions/terminate/{id}",
            post(controller::sessions_terminate),
        )
        .route_layer(middleware::from_fn(auth_guard::authenticated));

    public.merge(authenticated)
}
