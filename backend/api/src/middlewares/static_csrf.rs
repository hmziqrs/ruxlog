use std::env;

use crate::error::CsrfError;
use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{debug, instrument, warn};

pub fn get_static_csrf_key() -> String {
    let key = env::var("CSRF_KEY").unwrap_or_else(|_| "ultra-instinct-goku".to_string());
    return key;
}

#[instrument(skip(req, next), fields(token_present, decode_status, result, path))]
pub async fn csrf_guard(req: Request, next: Next) -> Result<Response, CsrfError> {
    // Exempt OAuth callback routes from CSRF check
    // OAuth uses the 'state' parameter for CSRF protection
    let path = req.uri().path();
    tracing::Span::current().record("path", path);

    if path.starts_with("/auth/google/v1/callback") || path.starts_with("/auth/google/v1/login") {
        debug!("Skipping CSRF check for OAuth route: {}", path);
        tracing::Span::current().record("result", "oauth_exempted");
        return Ok(next.run(req).await);
    }

    if let Some(token) = req.headers().get("csrf-token") {
        tracing::Span::current().record("token_present", true);
        debug!("CSRF token present in request");
        if let Ok(token_str) = token.to_str() {
            use base64::prelude::*;

            let parsed_token = BASE64_STANDARD.decode(token_str);
            match parsed_token {
                Ok(parsed_token) => {
                    tracing::Span::current().record("decode_status", "success");
                    let parsed_token = String::from_utf8(parsed_token);
                    match parsed_token {
                        Ok(parsed_token) => {
                            if parsed_token != get_static_csrf_key() {
                                warn!("CSRF token mismatch");
                                tracing::Span::current().record("result", "token_mismatch");
                                return Err(CsrfError::TokenMismatch);
                            }

                            debug!("CSRF token validated successfully");
                            tracing::Span::current().record("result", "valid");
                            Ok(next.run(req).await)
                        }
                        Err(_) => {
                            warn!("CSRF token UTF-8 decode failed");
                            tracing::Span::current().record("result", "invalid_utf8");
                            Err(CsrfError::InvalidUtf8)
                        }
                    }
                }
                Err(_) => {
                    warn!("CSRF token base64 decode failed");
                    tracing::Span::current().record("decode_status", "failed");
                    tracing::Span::current().record("result", "decode_error");
                    Err(CsrfError::InvalidBase64)
                }
            }
        } else {
            warn!("CSRF token header not valid string");
            tracing::Span::current().record("decode_status", "not_string");
            tracing::Span::current().record("result", "invalid_header");
            Err(CsrfError::InvalidHeader)
        }
    } else {
        warn!("CSRF token missing from request");
        tracing::Span::current().record("token_present", false);
        tracing::Span::current().record("result", "missing");
        Err(CsrfError::MissingToken)
    }
}

pub async fn test(req: Request, next: Next) -> Response {
    next.run(req).await
}
