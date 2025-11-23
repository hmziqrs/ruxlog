use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{instrument, warn};

use crate::error::CorsError;

/// Guard that rejects requests from origins not present in the configured
/// CORS allowlist, returning a standardized error response.
#[instrument(skip(req, next), fields(origin))]
pub async fn origin_guard(req: Request, next: Next) -> Result<Response, CorsError> {
    let origin_header = match req.headers().get(axum::http::header::ORIGIN) {
        None => {
            // Non-CORS or same-origin request; nothing to enforce here.
            return Ok(next.run(req).await);
        }
        Some(header) => header,
    };

    let origin_str = origin_header.to_str().unwrap_or("<invalid>").to_string();
    tracing::Span::current().record("origin", &*origin_str);

    let allowed_origins = crate::utils::cors::get_allowed_origins();
    let is_allowed = allowed_origins
        .iter()
        .any(|allowed| allowed == origin_header);

    if is_allowed {
        Ok(next.run(req).await)
    } else {
        warn!(origin = %origin_str, "Origin not allowed by CORS");
        Err(CorsError::OriginNotAllowed { origin: origin_str })
    }
}
