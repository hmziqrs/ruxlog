use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::json;
use std::env;

lazy_static! {
    static ref BLOCKED_ROUTES: Vec<Regex> = {
        let patterns = vec![
            r"^/admin/seed/v1",
            r"^/auth/v1/register$",
            //
            r"^/forgot_password/v1/request$",
            r"^/forgot_password/v1/verify$",
            r"^/forgot_password/v1/reset$",
            //
            r"^/email_verification/v1/verify$",
            r"^/email_verification/v1/resend$",
        ];
        patterns
            .into_iter()
            .map(|p| Regex::new(p).unwrap())
            .collect()
    };
}

pub async fn block_routes(req: Request, next: Next) -> Result<Response, Response> {
    // Get path from request
    let path = req.uri().path();

    // Check if we're in development mode
    let is_development =
        env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "development";

    // If we're in development, allow all routes
    if is_development {
        return Ok(next.run(req).await);
    }

    // Check if the path matches any blocked pattern
    for pattern in BLOCKED_ROUTES.iter() {
        if pattern.is_match(path) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "access_denied",
                    "message": "This route is currently unavailable"
                })),
            )
                .into_response());
        }
    }

    Ok(next.run(req).await)
}
