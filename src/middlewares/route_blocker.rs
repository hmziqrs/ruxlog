use crate::error::{ErrorCode, ErrorResponse};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use tracing::{debug, warn};

lazy_static! {
    static ref BLOCKED_ROUTES: Vec<Regex> = {
        let patterns = vec![
            r"^/admin/seed/v1",
            r"^/auth/v1/register$",
            r"^/forgot_password/v1/request$",
            r"^/forgot_password/v1/verify$",
            r"^/forgot_password/v1/reset$",
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
    let path = req.uri().path();

    let is_development =
        env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "development";

    if is_development {
        debug!(path, "Route blocker disabled in development mode");
        return Ok(next.run(req).await);
    }

    for pattern in BLOCKED_ROUTES.iter() {
        if pattern.is_match(path) {
            warn!(
                path,
                pattern = pattern.as_str(),
                "Route blocked by route_blocker middleware"
            );
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("This route is currently unavailable")
                .into_response());
        }
    }

    Ok(next.run(req).await)
}
