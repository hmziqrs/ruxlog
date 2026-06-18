//! `/csrf/v1/generate` — issues the per-session CSRF token (plan Phase 5a).
//!
//! Returns `token = base64(HMAC-SHA256(csrf_signing_key, session_id))`. The token
//! is bound to the caller's session: it is only valid for the exact session that
//! received it, and rotates when the session rotates (e.g. on login).
//!
//! For a brand-new session the id does not exist yet (tower-sessions assigns it
//! at response-time save), so this handler *bootstraps* the session first: it
//! inserts a marker (which marks the session non-empty + modified, so the
//! `SessionManagerLayer` will set the cookie on the way out) and calls
//! `session.save()` to materialize the id here and now. The client therefore
//! receives both the session cookie and the matching token in one response, and
//! login stays CSRF-protected (no login exemption is needed).

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tower_sessions::Session;

use crate::{
    error::{ErrorCode, ErrorResponse},
    middlewares::static_csrf::compute_csrf_token,
};

pub async fn generate(session: Session) -> Result<impl IntoResponse, ErrorResponse> {
    // Bootstrap a new session so we have an id to bind the token to. For an
    // existing session this is a cheap no-op pass-through.
    if session.id().is_none() {
        // Marker makes the session non-empty + modified → the layer persists it
        // and sets the cookie. A failure here is best-effort (save() below is
        // the load-bearing step).
        let _ = session.insert("csrf_issued", true).await;

        // Materializes session.id() (calls store.create for a new session) and
        // writes the record. If the store (Redis) is unreachable, we cannot
        // issue a token — fail closed with 500.
        session
            .save()
            .await
            .map_err(|_| ErrorResponse::new(ErrorCode::InternalServerError))?;
    }

    let Some(id) = session.id() else {
        // Should be unreachable after save(); guard regardless.
        return Err(ErrorResponse::new(ErrorCode::InternalServerError));
    };

    let token = compute_csrf_token(&id.to_string());

    Ok((
        StatusCode::OK,
        Json(json!({"message": "csrf token generated successfully", "token": token})),
    ))
}
